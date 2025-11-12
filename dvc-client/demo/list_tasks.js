const ProverClient = require('../src/prover_client');

async function main() {
  try {
    const api = new ProverClient();
    // this will get all the tasks
    const tasks = await api.listTasks();
    console.log('Tasks:', tasks);
  } catch (error) {
    console.error('Error fetching programs:', error);
  }
}

main();
