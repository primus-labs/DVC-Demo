const ProverClient = require('../src/prover_client');

async function main() {
  try {
    const api = new ProverClient();
    // set the task id return by submitTask
    const taskId = "2116be86-f69b-4bc5-a4ef-b46254243f22";
    const result = await api.getResult(taskId);
    console.log('Task submitted with environment:', result);
  } catch (error) {
    console.error('Error submitting task:', error);
  }
}

main();
