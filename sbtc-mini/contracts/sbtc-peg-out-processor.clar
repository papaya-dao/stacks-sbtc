(define-constant peg-out-state-requested 0x00)
(define-constant peg-out-state-fulfilled 0x01)

(define-data-var sbtc-token-burnchain-lock-time uint u2100)

(define-constant err-token-lock-failed (err u700))
(define-constant err-token-unlock-failed (err u701))
(define-constant err-unknown-peg-out-request (err u702))
(define-constant err-peg-out-not-requested (err u704))
(define-constant err-wrong-destination (err u705))
(define-constant err-unacceptable-expiry-height (err u706))
(define-constant err-wrong-value (err u707))
(define-constant err-unknown-destination-type (err u708))

(define-read-only (is-protocol-caller)
	(contract-call? .sbtc-controller is-protocol-caller contract-caller)
)

;; --- Protocol functions

;; #[allow(unchecked_data)]
(define-public (protocol-set-burnchain-lock-time (new-lock-time uint))
	(begin
		(try! (is-protocol-caller))
		(ok (var-set sbtc-token-burnchain-lock-time new-lock-time))
	)
)

;; --- Public functions

(define-read-only (get-burnchain-lock-time)
	(var-get sbtc-token-burnchain-lock-time)
)

(define-read-only (extract-request-data (tx (buff 4096)) (p2tr-unlock-script (buff 128)))
	;; It verifies the tapscript is the expected format.
	;; - "before burn height N, address X can spend, or else Y can spend"
	;; - check the expiry to make sure there is enough time to fulfil it
	;; - check if the script corresponds to the witness program in the tx

	;; Extract data from the Bitcoin transaction/tapscript:
	;; - The total BTC value requested to be pegged out, in sats
	;; - The principal pegging out
	;; - The burnchain peg-out expiry height

	;; To retrieve the principal of the entity pegging out (sender):
	;; message = something like amount + recipient scriptPubkey + nonce
	;; signature = embedded somewhere in the tapscript
	;; (principal-of? (unwrap! (secp256k1-recover? message signature) err-recovery-failed))

	;; make the type checker happy
	(if true (ok {
		sender: 'ST000000000000000000002AMW42H,
		destination: { version: 0x00, hashbytes: 0x0011223344556699001122334455669900112233445566990011223344556699},
		value: u100,
		expiry-burn-height: (+ burn-block-height (get-burnchain-lock-time))
		})
		(err u999999)
		)
)

(define-public (register-peg-out-request 
	(burn-height uint)
	(tx (buff 4096))
	(p2tr-unlock-script (buff 128))
	(header (buff 80))
	(tx-index uint)
	(tree-depth uint)
	(wproof (list 14 (buff 32)))
	(ctx (buff 1024))
	(cproof (list 14 (buff 32)))
	)
	(let (
		;; check if the tx was mined (todo: segwit wtxid)
		;; #[filter(tx)]
		(burn-wtxid (try! (contract-call? .clarity-bitcoin was-segwit-tx-mined-compact burn-height tx header tx-index tree-depth wproof 0x 0x ctx cproof)))
		;; get the peg out data
		;; #[filter(ts)]
		(peg-out-data (try! (extract-request-data tx p2tr-unlock-script)))
		)
		(asserts! false (err u99999999))
		;; There are still open questions about this part of the API.
		;; We can submit the P2TR funding transaction along with unlock script
		;; and store it, but it seems quite hard to verify that the unlock
		;; script can actually spend the P2TR output in Clarity. We have to
		;; derive the witness program and compare it with the one in the 
		;; transaction.

		;; check if the tx has not been processed before and if it
		;; reached the minimum amount of confirmations.
		(try! (contract-call? .sbtc-registry assert-new-burn-wtxid-and-height burn-wtxid burn-height))
		;; check that the expiry height is acceptable
		(asserts! (>= (get expiry-burn-height peg-out-data) (+ burn-block-height (get-burnchain-lock-time))) err-unacceptable-expiry-height)
		;; lock sender's the tokens
		(unwrap! (contract-call? .sbtc-token protocol-lock (get value peg-out-data) (get sender peg-out-data)) err-token-lock-failed)
		;; insert the request, returns the peg-out request-id
		(contract-call? .sbtc-registry insert-peg-out-request (get value peg-out-data) (get sender peg-out-data) (get expiry-burn-height peg-out-data) (get destination peg-out-data) p2tr-unlock-script)
	)
)

;; Bitcoin transactions must not contain more than 8 outputs.
(define-read-only (extract-total-destination-value (outs (list 8 { value: uint, scriptPubKey: (buff 128) })) (destination-scriptpubkey (buff 128)))
	(+
		(match (element-at? outs u0) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
		(match (element-at? outs u1) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
		(match (element-at? outs u2) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
		(match (element-at? outs u3) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
		(match (element-at? outs u4) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
		(match (element-at? outs u5) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
		(match (element-at? outs u6) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
		(match (element-at? outs u7) out (if (is-eq (get scriptPubKey out) destination-scriptpubkey) (get value out) u0) u0)
	)
)

(define-public (relay-peg-out-fulfilment
	(peg-out-request-id uint)
	(burn-height uint)
	(tx (buff 4096))
	(header (buff 80))
	(tx-index uint)
	(tree-depth uint)
	(wproof (list 14 (buff 32)))
	(witness-merkle-root (buff 32))
	(witness-reserved-data (buff 32))
	(witness-input-index uint)
	(ctx (buff 1024))
	(cproof (list 14 (buff 32)))
	)
	(let (
		;; Check if the tx was mined and get the parsed tx.
		(burn-tx (try! (contract-call? .sbtc-btc-tx-helper was-segwit-tx-mined burn-height tx header tx-index tree-depth wproof witness-merkle-root witness-reserved-data ctx cproof)))
		(burn-wtxid (get txid burn-tx))
		;; get the pending peg-out and mark it as settled.
		;; the call will fail if the request is no longer pending.
		(peg-out-request (try! (contract-call? .sbtc-registry get-and-settle-pending-peg-out-request peg-out-request-id peg-out-state-fulfilled)))
		;; Get the total value sent to the destination
		(total-value (extract-total-destination-value (get outs burn-tx) (unwrap! (contract-call? .sbtc-btc-tx-helper hashbytes-to-scriptpubkey (get destination peg-out-request)) err-unknown-destination-type)))
		)
		;; The protocol does not actually care who fulfilled the peg-out request.
		;; Anyone can pay the BTC, it does not have to come from the peg wallet.

		;; Check if the tx has not been processed before and if it
		;; reached the minimum amount of confirmations.
		(try! (contract-call? .sbtc-registry assert-new-burn-wtxid-and-height burn-wtxid burn-height))
		;; Check if the requested value was paid to the right destination address
		;; possible feature: allow transactions to partially peg out a request instead of
		;; all-or-nothing.
		(asserts! (>= total-value (get value peg-out-request)) err-wrong-value)
		;; burn the locked user tokens
		(contract-call? .sbtc-token protocol-burn-locked (get value peg-out-request) (get sender peg-out-request))
	)
)
