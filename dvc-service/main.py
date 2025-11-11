import time
from contextlib import asynccontextmanager
import asyncio, uuid, os, json, shutil, signal
from fastapi import FastAPI, UploadFile, Form, Query
from fastapi.responses import JSONResponse
from dotenv import load_dotenv
from pathlib import Path
import aiohttp
from datetime import datetime


# -------------------- Basic Configuration --------------------
load_dotenv()
MAX_CONCURRENCY = int(os.getenv("MAX_CONCURRENCY", 1))  # Max number of concurrent tasks
MAX_QUEUE_SIZE = int(os.getenv("MAX_QUEUE_SIZE", 10))  # Max queue size for tasks
PROGRAM_DIR = Path("programs")  # Directory to store uploaded programs
PROGRAM_DIR.mkdir(exist_ok=True)
METADATA_FILE = PROGRAM_DIR / "metadata.json"
TASKDATA_FILE = PROGRAM_DIR / "task_store.json"

SUCCINCT_PROVER_BIN = os.getenv("SUCCINCT_PROVER_BIN", "")
if not Path(SUCCINCT_PROVER_BIN).exists():
    print(f"Succinct prover:{SUCCINCT_PROVER_BIN} not exist!")
    exit(1)
print(f"Succinct prover: {SUCCINCT_PROVER_BIN}")

# If metadata file does not exist, create an empty one
if not METADATA_FILE.exists():
    METADATA_FILE.write_text("{}", encoding="utf-8")

TASKS = {}  # Stores task metadata: task_id -> {...}
TASK_QUEUE = asyncio.Queue()  # Queue for processing tasks

# Create FastAPI app
app = FastAPI()


# -------------------- Utility Functions --------------------
def save_tasks():
    """Saves the current task metadata to a file."""
    with open(TASKDATA_FILE, "w") as f:
        json.dump(TASKS, f, indent=2)


def load_tasks():
    """Loads task metadata from a file."""
    if os.path.exists(TASKDATA_FILE):
        with open(TASKDATA_FILE, "r") as f:
            data = json.load(f)
            for tid, t in data.items():
                # Ensure that tasks that were previously running or queued are treated as queued now
                if t["status"] in ["running", "queued"]:
                    t["status"] = "queued"
                TASKS[tid] = t


def save_metadata(meta):
    """Saves program metadata (including timestamps) to a file."""
    with open(METADATA_FILE, "w") as f:
        json.dump(meta, f, indent=2)


def load_metadata():
    """Loads program metadata from a file."""
    if METADATA_FILE.exists():
        with open(METADATA_FILE, "r") as f:
            return json.load(f)
    return {}


# -------------------- Worker --------------------
async def worker():
    """Worker function that processes tasks from the queue."""
    while True:
        task_id, cmd, callback, env_vars = await TASK_QUEUE.get()
        task = TASKS.get(task_id)
        if not task:
            TASK_QUEUE.task_done()
            continue

        TASKS[task_id]["status"] = "running"
        save_tasks()

        env = os.environ.copy()
        t_start = time.perf_counter()
        proof_fixture = "{}"
        try:
            output_dir = f"./proof_output/{task_id}"
            os.makedirs(output_dir, exist_ok=True)

            input_file = TASKS[task_id]["input_file"]
            elf_file = TASKS[task_id]["program_path"]

            _prover = TASKS[task_id]["prover"]
            if _prover == "succinct":
                BIN = SUCCINCT_PROVER_BIN
                # env["DEBUG"] = "1"
            else:
                raise Exception(f"Unsupported prover:{_prover}")

            # cmd = [BIN, "--execute", "--elf", elf_file, "--input", input_file, "--output-dir", output_dir]
            cmd = [BIN, "--prove", "--elf", elf_file, "--input", input_file, "--output-dir", output_dir]
            print("cmd", cmd, "\nenv", env_vars)
            # Run the program as a subprocess with the environment variables
            proc = await asyncio.create_subprocess_exec(
                *cmd, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE, env=env
            )
            TASKS[task_id]["pid"] = proc.pid
            stdout, stderr = await proc.communicate()

            if os.path.exists(f"{output_dir}/proof_fixture.json"):
                with open(f"{output_dir}/proof_fixture.json", "r", encoding="utf-8") as f:
                    proof_fixture = f.read()

            TASKS[task_id]["status"] = "done"
            TASKS[task_id]["result"] = stdout.decode() or stderr.decode()
        except Exception as e:
            TASKS[task_id]["status"] = "error"
            TASKS[task_id]["result"] = str(e)
        finally:
            t_end = time.perf_counter()
            TASKS[task_id]["proof_fixture"] = proof_fixture
            TASKS[task_id]["elapsed"] = f"{t_end - t_start:.6f}"
            save_tasks()
            TASK_QUEUE.task_done()
            if callback:
                # Notify the callback URL with task result
                asyncio.create_task(post_callback(callback, task_id, TASKS[task_id]))


async def post_callback(url, task_id, data):
    """Sends the task result to a callback URL."""
    try:
        async with aiohttp.ClientSession() as session:
            await session.post(url, json={"task_id": task_id, "data": data})
    except Exception as e:
        print(f"[callback error] {e}")


# -------------------- Lifespan Context --------------------
@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan manager to handle startup and shutdown logic."""
    # Startup
    load_tasks()
    for _ in range(MAX_CONCURRENCY):
        asyncio.create_task(worker())

    for tid, t in TASKS.items():
        if t["status"] == "queued":
            await TASK_QUEUE.put((tid, t["cmd"], t.get("callback"), t.get("env")))

    yield  # The app runs here


app = FastAPI(lifespan=lifespan)


# -------------------- Program Management --------------------
@app.post("/uploadProgram")
async def upload_program(
    file: UploadFile,
    prover: str = Form("succinct"),
    name: str = Form(""),
    version: str = Form(""),
    desc: str = Form(""),
):
    """
    Uploads a program file and stores it along with metadata, including upload timestamp.
    """
    if prover != "succinct":
        return JSONResponse(status_code=400, content={"error": "only support succinct now"})

    program_id = str(uuid.uuid4())
    path = PROGRAM_DIR / program_id

    # Save the uploaded file
    with open(path, "wb") as f:
        shutil.copyfileobj(file.file, f)

    # Set file permissions
    os.chmod(path, 0o755)

    # Get the current timestamp
    uploaded_at = datetime.now().isoformat()  # ISO 8601 timestamp

    # Load existing metadata and add the new program's metadata
    meta = load_metadata()
    meta[program_id] = {"prover": prover, "name": name, "version": version, "desc": desc, "uploaded_at": uploaded_at}

    # Save updated metadata
    save_metadata(meta)

    return {"program_id": program_id, "uploaded_at": uploaded_at}


@app.get("/listPrograms")
async def list_programs():
    """
    Returns a list of all uploaded programs along with their metadata.
    """
    return load_metadata()


# -------------------- Task Management --------------------
@app.post("/submitTask")
async def submit_task(
    program_id: str = Form(...), attestation_data: str = Form(""), callback: str = Form(None), env: str = Form(None)
):
    """
    Queues a task to run the specified program with arguments and environment variables.
    """
    if TASK_QUEUE.qsize() >= MAX_QUEUE_SIZE:
        return JSONResponse(status_code=429, content={"error": "queue_full"})

    program_path = PROGRAM_DIR / program_id
    if not program_path.exists():
        return JSONResponse(status_code=404, content={"error": f"program {program_id} not found in disk"})

    meta = load_metadata()
    if program_id not in meta:
        return JSONResponse(status_code=404, content={"error": f"program {program_id} not found in metadata"})
    prover = meta[program_id]["prover"]

    task_id = str(uuid.uuid4())

    # Prepare environment variables (if any)
    env_vars = {}
    if env:
        try:
            env_vars = json.loads(env)
        except json.JSONDecodeError:
            return JSONResponse(status_code=400, content={"error": "invalid_env_format, expect json string"})

    input_dir = f"./request_data"
    os.makedirs(input_dir, exist_ok=True)
    input_file = f"{input_dir}/{task_id}.json"
    with open(input_file, "w", encoding="utf-8") as f:
        f.write(attestation_data)

    cmd = ""  # placeholder
    TASKS[task_id] = {
        "status": "queued",
        "cmd": cmd,
        "prover": prover,
        "input_file": input_file,
        "result": None,
        "callback": callback,
        "env": env_vars,
        "program_path": str(program_path),
        "submitted_at": datetime.now().isoformat(),
    }
    save_tasks()

    await TASK_QUEUE.put((task_id, cmd, callback, env_vars))
    return {"task_id": task_id, "status": "queued"}


@app.get("/getResult")
async def get_result(task_id: str):
    """
    Retrieves the result of a task using the task ID.
    """
    task = TASKS.get(task_id)  # look from memory first
    if not task:
        return JSONResponse(status_code=404, content={"error": "not_found"})
    return task


@app.get("/listTasks")
async def list_tasks(status: str = Query(None)):
    """
    Lists tasks based on their status (queued, running, or done).
    """
    if status:
        return {tid: t for tid, t in TASKS.items() if t["status"] == status}
    return TASKS


@app.delete("/deleteTask")
async def delete_task(task_id: str):
    """
    Deletes a task by task ID.
    """
    if task_id in TASKS:
        task = TASKS[task_id]
        if task["status"] == "running" and "pid" in task:
            try:
                os.kill(task["pid"], signal.SIGTERM)
            except Exception:
                pass
        del TASKS[task_id]
        save_tasks()
        return {"deleted": task_id}
    return JSONResponse(status_code=404, content={"error": "task {task_id} not found"})


@app.post("/pauseTask")
async def pause_task(task_id: str):
    """
    Pauses a task that is in the 'queued' state.
    """
    for i in range(TASK_QUEUE.qsize()):
        tid, cmd, cb, env = await TASK_QUEUE.get()
        if tid == task_id:
            TASKS[tid]["status"] = "paused"
            save_tasks()
            return {"paused": task_id}
        else:
            await TASK_QUEUE.put((tid, cmd, cb, env))
    return JSONResponse(status_code=404, content={"error": "task {task_id} not found"})
