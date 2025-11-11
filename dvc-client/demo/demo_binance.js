const ccxt = require('ccxt');
require('dotenv').config();

const { saveToFile } = require('../src/utils')
const ZKTLSClient = require('../src/zktls_client');

function getBinanaceRequestParams() {
  const key = process.env.BINANCE_API_KEY;
  const secret = process.env.BINANCE_API_SECRET;
  if (!key || key === "" || !secret || secret === "") {
    throw new Error(`Please set BINANCE_API_KEY or BINANCE_API_SECRET in .env`);
  }
  const recvWindow = Number(process.env.BINANCE_RECV_WINDOW) || 60;

  const exchange = new ccxt['binance']({
    apiKey: key,
    secret: secret
  });

  let signParams = { recvWindow: recvWindow * 1000, omitZeroBalances: true };
  let origRequest = exchange.sign('account', 'private', 'GET', signParams);
  console.log("origRequest:", origRequest);

  const requests = [
    {
      url: origRequest.url,
      method: "GET",
      header: { ...origRequest.headers },
      body: "",
    },
  ];

  const responseResolves = [
    [
      {
        keyName: "hash-of-response",
        parseType: "json",
        parsePath: "$",
        op: "SHA256_EX"
      },
    ],
  ];
  return { requests, responseResolves };
}

async function main() {
  const client = new ZKTLSClient();
  const { requests, responseResolves } = getBinanaceRequestParams();
  const zkvmReqeustData = await client.doZKTLS(requests, responseResolves, {
    runZkvm: false,
    noProxy: false
  });
  // console.log("zkvmReqeustData:", JSON.stringify(zkvmReqeustData));
  if (zkvmReqeustData && zkvmReqeustData.attestationData) {
    saveToFile("binance-attestation.json", JSON.stringify(zkvmReqeustData.attestationData));
  }
}

main();
