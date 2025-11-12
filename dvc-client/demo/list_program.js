const ProverClient = require('../src/prover_client');

async function main() {
  try {
    const api = new ProverClient();
    // this will get all programs
    const programs = await api.listPrograms();
    console.log('Programs:', programs);
  } catch (error) {
    console.error('Error fetching programs:', error);
  }
}

main();
