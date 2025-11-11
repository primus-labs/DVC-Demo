const axios = require('axios');
const FormData = require('form-data');
const fs = require('fs');
require('dotenv').config();

class ProverClient {
  constructor(timeout = 10000) {
    const baseURL = process.env.DVC_SERVICE_URL;
    console.log(baseURL)
    if (!baseURL || baseURL === "") {
      throw new Error(`Please set DVC_SERVICE_URL in .env`);
    }
    this.client = axios.create({
      baseURL,
      timeout,
    });
  }
  async uploadProgram(filePath, name, version, desc, prover = 'succinct') {
    const form = new FormData();
    form.append('file', fs.createReadStream(filePath));
    form.append('prover', prover);
    form.append('name', name);
    form.append('version', version);
    form.append('desc', desc);

    try {
      const res = await this.client.post('/uploadProgram', form, {
        headers: form.getHeaders(),
      });
      return res.data;
    } catch (err) {
      console.error('❌ uploadProgram failed:', err.message);
      throw err;
    }
  }

  async listPrograms() {
    try {
      const res = await this.client.get('/listPrograms');
      return res.data;
    } catch (err) {
      console.error('❌ listPrograms failed:', err.message);
      throw err;
    }
  }

  async submitTask(programId, attestationData) {
    const form = new FormData();
    form.append('program_id', programId);
    form.append('attestation_data', attestationData);

    try {
      const res = await this.client.post('/submitTask', form, {
        headers: form.getHeaders(),
      });
      return res.data;
    } catch (err) {
      console.error('❌ submitTask failed:', err.message);
      throw err;
    }
  }

  async getResult(taskId) {
    try {
      const res = await this.client.get('/getResult', {
        params: { task_id: taskId },
      });
      return res.data;
    } catch (err) {
      console.error('❌ getResult failed:', err.message);
      throw err;
    }
  }

  async listTasks(status = null) {
    try {
      const res = await this.client.get('/listTasks', {
        params: { status },
      });
      return res.data;
    } catch (err) {
      console.error('❌ listTasks failed:', err.message);
      throw err;
    }
  }

  async deleteTask(taskId) {
    try {
      const res = await this.client.delete('/deleteTask', {
        data: { task_id: taskId },
      });
      return res.data;
    } catch (err) {
      console.error('❌ deleteTask failed:', err.message);
      throw err;
    }
  }

  async pauseTask(taskId) {
    try {
      const res = await this.client.post('/pauseTask', null, {
        params: { task_id: taskId },
      });
      return res.data;
    } catch (err) {
      console.error('❌ pauseTask failed:', err.message);
      throw err;
    }
  }
}

module.exports = ProverClient;
