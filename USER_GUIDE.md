
## User Guide

Imagine this scenario: You have a Binance account and need to prove that your ETH balance is greater than 0.1.

Where can we get the balances? We can use [Account information (USER_DATA)](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints#account-information-user_data) API to query the balances.

### Step 1: Generate Attestation Data

From the Binance API documentation, you can identify the **request URL** and **parameters**. Using these, generate the required request URL with **ccxt** (a library to integrate with CEX APIs). Then, following the [Primus Network](https://docs.primuslabs.xyz/primus-network/understand-primus-network) documentation, fill in the request parameters and specify how to parse the response. Finally, run **ZKTLS** with **Primus Network** to obtain the **attestation data**. A complete example can be found in: [demo_binance.js](./dvc-client/demo/demo_binance.js).

> **Note:** You should configure your Binance account’s `API_KEY` and `API_SECRET` in the `.env` file.


<br/>

**Structure of Attestation Data**

The generated **attestation data** will be used in subsequent **zkVM verification**. It consists of three parts (see the [sample](./testdata/binance-attestation.json) for more details):

```json
{
  "public_data": [],  // The public content of the attestation
  "private_data": {},  // Private data, visible only to the user
  "verification_type": "HASH_COMPARISON"  // Verification type, here it’s hash comparison
}
```

### Step 2: Write the Program Based on Business Logic

**zkVM** is a **Rust program**. Let’s go through how to write it according to your business requirements.

<br/>

**(1). Verification Step — Mandatory**

First, validate the **signature**, **attestor**, and **data source URL** from the previously generated attestation.

Then, based on `"verification_type": "HASH_COMPARISON"`, check whether the hash values match.

> **This step is mandatory for all business operations.**


<br/>

**(2). Implement the Business Logic**

Next, extract the required business fields. For example, from the `balances` array:

  1. Check whether `asset` equals **ETH**.
  2. If yes, sum the `free` and `locked` amounts.
  3. Compare this total with the threshold **0.1**.
  4. If the total exceeds **0.1**, verification passes. Otherwise, verification fails.

For a complete example, refer to [dvc-succinct-program/src/binance.rs](./dvc-succinct-program/src/binance.rs). 

<br/>

**(3). Compile and Upload the Program**

Compile the Rust code into an **ELF file**, as described in the [dvc-succinct-program/README.md](./dvc-succinct-program/README.md).

Then, refer [upload program](./dvc-client/demo/upload_program.js) to upload the ELF binary to the **TEE service** running the prover and return a **program ID**.


## Step 3: Submit the Task

Refer [submit a task](./dvc-client/demo/submit_task.js) to send the generated **attestation data**(Step 1) and the **program ID**(Step 2) to the **TEE service**. You will immediately receive a **task ID**.

The TEE service will execute the prover using the uploaded **ELF Program**. Once the task completes, you can retrieve the result by [querying](./dvc-client/demo/get_result.js) with the **task ID**.

> **Tip:** Step 3 can be combined with Step 1 - allowing you to **generate the attestation data**, **submit the task**, and **retrieve the result** in one unified workflow.

