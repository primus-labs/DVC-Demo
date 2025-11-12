const fs = require('fs');
const ProverClient = require('../src/prover_client');

async function main() {
  try {
    const api = new ProverClient();
    // set the program id you have uploaded, and the attestation data path
    const programId = "c3929a34-e3ab-4275-9d89-9ec4f85bd986";
    const attestation_data = fs.readFileSync("../testdata/binance-attestation.json", encoding = "utf-8");
    const result = await api.submitTask(programId, attestation_data);
    console.log('Task submitted:', result);
  } catch (error) {
    console.error('Error submitting task:', error);
  }
}

main();
