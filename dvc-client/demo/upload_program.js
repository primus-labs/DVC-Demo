const ProverClient = require('../src/prover_client');

async function main() {
  try {
    const api = new ProverClient();
    // set your own elf binary path and some meta info
    const elf = "../dvc-succinct-program/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/dvc-succinct-program";
    const result = await api.uploadProgram(elf, 'My Program', '1.0', 'Program description');
    console.log('Program uploaded, ID:', result.program_id);
  } catch (error) {
    console.error('Error during upload:', error);
  }
}

main();
