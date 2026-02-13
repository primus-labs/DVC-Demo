const ccxt = require('ccxt');
require('dotenv').config();

const { saveToFile } = require('../src/utils')
const ZKTLSClient = require('../src/zktls_client');

function getOkxRequestParams() {
  const requests = [
    {
      url: "https://www.okx.com/api/v5/public/instruments?instType=SPOT&instId=BTC-USD",
      method: "GET",
      header: {},
      body: "",
    },
  ];

  const responseResolves = [
    [
      {
        keyName: "grumpkin-commitment",
        parseType: "json",
        parsePath: "$.data[0].baseCcy",
        op: "REVEAL_GRUMPKIN_COMMITMENT"
      }
    ],
  ];

  return { requests, responseResolves };
}

async function main() {
  const client = new ZKTLSClient();
  const { requests, responseResolves } = getOkxRequestParams();
  const zkvmReqeustData = await client.doZKTLS(requests, responseResolves, {
    verifyVersion: "2",
    algorithmType: "proxytls",
    runZkvm: false,
    noProxy: false
  });
  // console.log("zkvmReqeustData:", JSON.stringify(zkvmReqeustData));
  if (zkvmReqeustData && zkvmReqeustData.attestationData) {
    saveToFile("okx-grumpkin-commitment-attestation.json", JSON.stringify(zkvmReqeustData.attestationData));
  }
}

main();
