use aztec::macros::aztec;

/**
  * Binance Balance Verifier Contract
  *
  * This contract verifies Primus zkTLS attestations for Binance API data.
  * It supports two verification methods:
  * 1) Commitment-based attestations (Grumpkin curve)
  * 2) Hashing-based attestations (SHA256)
  *
  * Business logic: Verify that a user's ETH balance on Binance exceeds a threshold.
  */
#[aztec]
pub contract BinanceVerifier {
    use att_verifier_lib::{verify_attestation_comm, verify_attestation_hashing};
    use aztec::macros::{events::event, functions::{external, initializer}};
    use aztec::macros::storage::storage;
    use aztec::protocol::address::AztecAddress;
    use aztec::state_vars::{PublicImmutable, PublicMutable};
    use std::embedded_curve_ops::EmbeddedCurvePoint;

    // Configuration constants
    global MAX_URL_LEN: u32 = 128;
    global MAX_PLAINTEXT_LEN: u32 = 100;
    global NUM_RESPONSE_RESOLVE: u32 = 1;
    global NUM_REQUEST_URLS: u32 = 1;
    global NUM_ALLOWED_URLS: u32 = 1;
    global MAX_COMMS: u32 = 2;

    #[event]
    struct BalanceVerifiedEvent {
        sender: AztecAddress,
        contract_address: AztecAddress,
        id: Field,
        eth_balance: u128,
    }

    #[storage]
    struct Storage<Context> {
        admin: PublicMutable<AztecAddress, Context>,
        allowed_url_hashes: PublicMutable<[Field; NUM_ALLOWED_URLS], Context>,
        H: PublicImmutable<EmbeddedCurvePoint, Context>,
    }

    #[external("public")]
    fn set_admin(new_admin: AztecAddress) {
        assert(self.storage.admin.read().eq(self.msg_sender()), "caller is not admin");
        self.storage.admin.write(new_admin);
    }

    #[initializer]
    #[external("public")]
    fn constructor(admin: AztecAddress, allowed_url_hashes: [Field; NUM_ALLOWED_URLS], H: EmbeddedCurvePoint) {
        assert(!admin.is_zero(), "invalid admin");
        self.storage.admin.write(admin);
        self.storage.allowed_url_hashes.write(allowed_url_hashes);
        self.storage.H.initialize(H);
    }

    #[external("public")]
    fn update_allowed_url_hashes(allowed_url_hashes: [Field; NUM_ALLOWED_URLS]) {
        assert(self.storage.admin.read().eq(self.msg_sender()), "caller is not admin");
        self.storage.allowed_url_hashes.write(allowed_url_hashes);
    }

    #[external("public")]
    fn check_urls_emit_event(
        sender: AztecAddress,
        contract_address: AztecAddress,
        id: Field,
        eth_balance: u128,
        allowed_url_matches_hashes: [Field; NUM_REQUEST_URLS],
    ) {
        let allowed_url_hashes: [Field; NUM_ALLOWED_URLS] = self.storage.allowed_url_hashes.read();
        for i in 0..NUM_REQUEST_URLS {
            let check_hash = allowed_url_matches_hashes[i];
            let mut found = false;
            for j in 0..NUM_ALLOWED_URLS {
                if allowed_url_hashes[j] == check_hash {
                    found = true;
                }
            }
            assert(found);
        }

        self.emit(BalanceVerifiedEvent { sender, contract_address, id, eth_balance });
    }

    /**
      * Verify hash-based attestation and check ETH balance threshold.
      *
      * The attestation should contain Binance account balance data.
      * This function verifies:
      * 1. Signature validity
      * 2. URL is from allowed Binance API
      * 3. Hash of content matches
      * 4. ETH balance > threshold (business logic)
      */
    #[external("private")]
    fn verify_hash(
        public_key_x: [u8; 32],
        public_key_y: [u8; 32],
        hash: [u8; 32],
        signature: [u8; 64],
        request_urls: [BoundedVec<u8, MAX_URL_LEN>; NUM_REQUEST_URLS],
        allowed_urls: [BoundedVec<u8, MAX_URL_LEN>; NUM_ALLOWED_URLS],
        data_hashes: [[u8; 32]; NUM_RESPONSE_RESOLVE],
        contents: [BoundedVec<u8, MAX_PLAINTEXT_LEN>; NUM_RESPONSE_RESOLVE],
        id: Field,
        eth_balance: u128,        // Parsed ETH balance from contents
        threshold: u128,          // Minimum required balance (e.g., 0.1 ETH = 100000)
    ) -> bool {
        // Verify attestation
        let allowed_url_matches_hashes: [Field; NUM_REQUEST_URLS] = verify_attestation_hashing(
            public_key_x,
            public_key_y,
            hash,
            signature,
            request_urls,
            allowed_urls,
            data_hashes,
            contents,
        );

        // Business logic: Check ETH balance > threshold
        // eth_balance is in units of 0.000001 ETH (6 decimal places)
        // e.g., 0.1 ETH = 100000
        assert(eth_balance > threshold, "ETH balance below threshold");

        BinanceVerifier::at(self.address)
            .check_urls_emit_event(
                self.msg_sender(),
                self.address,
                id,
                eth_balance,
                allowed_url_matches_hashes,
            )
            .enqueue(self.context);

        true
    }

    /**
      * Verify commitment-based attestation and check ETH balance threshold.
      */
    #[external("private")]
    fn verify_comm(
        public_key_x: [u8; 32],
        public_key_y: [u8; 32],
        hash: [u8; 32],
        signature: [u8; 64],
        request_urls: [BoundedVec<u8, MAX_URL_LEN>; NUM_REQUEST_URLS],
        allowed_urls: [BoundedVec<u8, MAX_URL_LEN>; NUM_ALLOWED_URLS],
        coms_per_group: [BoundedVec<EmbeddedCurvePoint, MAX_COMMS>; NUM_RESPONSE_RESOLVE],
        rnds_per_group: [BoundedVec<Field, MAX_COMMS>; NUM_RESPONSE_RESOLVE],
        msgs_chunks_per_group: [BoundedVec<Field, MAX_COMMS>; NUM_RESPONSE_RESOLVE],
        H: EmbeddedCurvePoint,
        id: Field,
        eth_balance: u128,
        threshold: u128,
    ) -> bool {
        let allowed_url_matches_hashes: [Field; NUM_REQUEST_URLS] = verify_attestation_comm(
            public_key_x,
            public_key_y,
            hash,
            signature,
            request_urls,
            allowed_urls,
            coms_per_group,
            rnds_per_group,
            msgs_chunks_per_group,
            H,
        );

        assert(eth_balance > threshold, "ETH balance below threshold");

        BinanceVerifier::at(self.address)
            .check_values_emit_event(
                self.msg_sender(),
                self.address,
                id,
                eth_balance,
                allowed_url_matches_hashes,
                H,
            )
            .enqueue(self.context);

        true
    }

    #[external("public")]
    fn check_values_emit_event(
        sender: AztecAddress,
        contract_address: AztecAddress,
        id: Field,
        eth_balance: u128,
        allowed_url_matches_hashes: [Field; NUM_REQUEST_URLS],
        H: EmbeddedCurvePoint,
    ) {
        let allowed_url_hashes: [Field; NUM_ALLOWED_URLS] = self.storage.allowed_url_hashes.read();
        for i in 0..NUM_REQUEST_URLS {
            let check_hash = allowed_url_matches_hashes[i];
            let mut found = false;
            for j in 0..NUM_ALLOWED_URLS {
                if allowed_url_hashes[j] == check_hash {
                    found = true;
                }
            }
            assert(found);
        }

        let stored_H: EmbeddedCurvePoint = self.storage.H.read();
        assert(H == stored_H);

        self.emit(BalanceVerifiedEvent { sender, contract_address, id, eth_balance });
    }
}