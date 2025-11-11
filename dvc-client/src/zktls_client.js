const { PrimusNetwork } = require('@primuslabs/network-core-sdk/dist');
const { ethers } = require('ethers');
require('dotenv').config();

const { sleepMs } = require('./utils')


class ZKTLSClient {
  constructor() {
    this.primusNetwork = new PrimusNetwork();
  }


  /**
   * Main entry: perform ZKTLS attestation and task flow
   */
  async doZKTLS(requests, responseResolves, options = {}) {
    const opts = this._getDefaultOptions(options);

    this._validateInput(requests, responseResolves);
    await this._validateEnvVars();

    const { PRIVATE_KEY, CHAIN_ID, RPC_URL } = process.env;
    const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
    const wallet = new ethers.Wallet(PRIVATE_KEY, provider);
    const attestParams = { address: '0x810b7bacEfD5ba495bB688bbFD2501C904036AB7' };

    const startTime = Date.now();
    try {
      await this._initializePrimusNetwork(wallet, CHAIN_ID);

      const submitResult = await this._submitZktlsTaskWithRetry(opts, attestParams);
      const attestResult = await this._attestWithRetry(
        requests,
        responseResolves,
        opts,
        attestParams,
        submitResult
      );
      const taskResult = await this._verifyAndPollTaskResultWithRetry(attestResult);

      const zkVmRequestData = await this._prepareZkVmRequestData(taskResult, attestResult);
      console.log(`✅ Total execution time: ${Date.now() - startTime}ms`);
      return zkVmRequestData;
    } catch (err) {
      throw new Error(`ZKTLS execution failed: ${err.message || err}`);
    }
  }

  /**
   * Default options for ZKTLS attestation
   */
  _getDefaultOptions(options) {
    const defaults = {
      sslCipher: 'ECDHE-RSA-AES128-GCM-SHA256',
      algorithmType: 'mpctls',
      specialTask: undefined,
      noProxy: true,
      runZkvm: true,
      requestParamsCallback: undefined,
    };
    return { ...defaults, ...options };
  }

  /**
   * Validate request and response array inputs
   */
  _validateInput(requests, responseResolves) {
    if (
      !Array.isArray(requests) ||
      requests.length !== responseResolves.length ||
      requests.length === 0
    ) {
      throw new Error("Invalid 'requests' or 'responseResolves' size");
    }
  }

  /**
   * Ensure all required environment variables are present
   */
  async _validateEnvVars() {
    const required = ['PRIVATE_KEY', 'CHAIN_ID', 'RPC_URL'];
    for (const key of required) {
      if (!process.env[key]) {
        throw new Error(`Missing environment variable: ${key}`);
      }
    }
  }

  /**
   * Initialize PrimusNetwork SDK
   */
  async _initializePrimusNetwork(wallet, chainId) {
    try {
      console.log('🚀 Initializing PrimusNetwork...');
      const initResult = await this.primusNetwork.init(wallet, +chainId, 'native');
      console.log('✅ PrimusNetwork initialized:', initResult);
    } catch (err) {
      throw new Error(`PrimusNetwork init failed: ${err.message || err}`);
    }
  }

  /**
   * Submit ZKTLS task with retry and exponential backoff
   */
  async _submitZktlsTaskWithRetry(opts, attestParams, maxRetries = 5, baseDelay = 1000) {
    let attempt = 0;
    const start = Date.now();

    console.log('📝 Submitting ZKTLS task...');
    while (true) {
      try {
        const result = await this.primusNetwork.submitTask(attestParams);
        console.log(`✅ submitTask ZKTLS done (${Date.now() - start}ms):`, result);
        return result;
      } catch (err) {
        attempt++;
        console.warn(`⚠️ submitTask ZKTLS attempt ${attempt} failed:`, err?.message || err);
        if (attempt > maxRetries) throw new Error(`submitTask ZKTLS failed after ${maxRetries} retries`);
        await sleepMs(baseDelay * 2 ** (attempt - 1));
      }
    }
  }

  /**
   * Run attestation with retries
   */
  async _attestWithRetry(requests, responseResolves, opts, attestParams, submitResult, maxRetries = 3, baseDelay = 1000) {
    let attempt = 0;
    const start = Date.now();

    console.log('⚙️ Running attestation...');
    while (true) {
      try {
        let reqs = requests;
        let resps = responseResolves;
        if (opts.requestParamsCallback) {
          const cb = opts.requestParamsCallback();
          reqs = cb.requests;
          resps = cb.responseResolves;
        }

        const fullParams = {
          ...attestParams,
          ...submitResult,
          requests: reqs,
          responseResolves: resps,
          sslCipher: opts.sslCipher,
          attMode: { algorithmType: opts.algorithmType },
          specialTask: opts.specialTask,
          noProxy: opts.noProxy,
          getAllJsonResponse: 'true',
        };

        const result = await this.primusNetwork.attest(fullParams, 5 * 60 * 1000);
        if (!result?.[0]?.attestation) throw new Error('invalid attestation result');
        console.log(`✅ attest done (${Date.now() - start}ms):`, result);
        return result;
      } catch (err) {
        attempt++;
        console.warn(`⚠️ attest attempt ${attempt} failed:`, err?.message || err);
        if (attempt > maxRetries) throw new Error(`attest failed after ${maxRetries} retries`);
        await sleepMs(baseDelay * 2 ** (attempt - 1));
      }
    }
  }

  /**
   * Verify and poll task result with retries
   */
  async _verifyAndPollTaskResultWithRetry(attestResult, maxRetries = 5, baseDelay = 1000) {
    let attempt = 0;
    const start = Date.now();

    console.log('🔍 Verifying and polling task result...');
    while (true) {
      try {
        const result = await this.primusNetwork.verifyAndPollTaskResult({
          taskId: attestResult[0].taskId,
          reportTxHash: attestResult[0].reportTxHash,
        });
        console.log(`✅ Verification done (${Date.now() - start}ms):`, result);
        return result;
      } catch (err) {
        attempt++;
        console.warn(`⚠️ verifyAndPollTaskResult attempt ${attempt} failed:`, err?.message || err);
        if (attempt > maxRetries) throw new Error(`verifyAndPollTaskResult failed after ${maxRetries} retries`);
        await sleepMs(baseDelay * 2 ** (attempt - 1));
      }
    }
  }

  /**
   * Prepare final zkVM attestation data
   */
  async _prepareZkVmRequestData(taskResult, attestResult) {
    const taskId = attestResult[0].taskId;
    const plainResponse = this.primusNetwork.getAllJsonResponse(taskId);

    if (!plainResponse) throw new Error('Unable to get plain JSON response');

    return {
      attestationData: {
        verification_type: 'HASH_COMPARISON',
        public_data: attestResult,
        private_data: { plain_json_response: plainResponse },
      },
      requestid: taskId,
    };
  }

}

module.exports = ZKTLSClient;
