(define-constant peg-out-state-reclaimed 0x02)

(define-constant err-peg-out-not-epxired (err u803))

;; unlocks the sBTC tokens after expiry
(define-public (reclaim-locked-tokens (peg-out-request-id uint))
	(let (
		;; get the pending peg-out and mark it as settled.
		;; the call will fail if the request is no longer pending.
		(peg-out-request (try! (contract-call? .sbtc-registry get-and-settle-pending-peg-out-request peg-out-request-id peg-out-state-reclaimed)))
		)
		;; check if the peg-out request has expired (pending check is done above)
		(asserts! (<= (get expiry-burn-height peg-out-request) burn-block-height) err-peg-out-not-epxired)
		;; unlock the locked user tokens
		(contract-call? .sbtc-token protocol-unlock (get value peg-out-request) (get sender peg-out-request))
	)
)
