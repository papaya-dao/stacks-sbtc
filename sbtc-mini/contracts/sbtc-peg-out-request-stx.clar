(define-constant sbtc-token-burnchain-lock-time u2100)
(define-constant max-uint128 u340282366920938463463374607431768211455)

(define-constant err-token-lock-failed (err u700))
(define-constant err-cannot-set-allowance-for-self (err u701))
(define-constant err-operator-not-allowed (err u702))

(define-constant err-no-sponsor (err u798))
(define-constant err-invalid-sender (err u799))

(define-read-only (get-expiry-burn-height)
	(+ burn-block-height sbtc-token-burnchain-lock-time)
)

(define-map allowances {sender: principal, operator: principal} uint)

(define-read-only (get-allowance (sender principal) (operator principal))
	(default-to u0 (map-get? allowances {sender: sender, operator: operator}))
)

;; #[allow(unchecked_data)]
(define-public (set-allowance (operator principal) (allowance uint))
	(begin
		(asserts! (not (is-eq contract-caller operator)) err-cannot-set-allowance-for-self)
		(ok (map-set allowances {sender: contract-caller, operator: operator} allowance))
	)
)

(define-private (is-allowed-operator-and-deduct (sender principal) (operator principal) (amount uint))
	(begin
		(asserts! (not (is-eq sender operator)) true)
		(let ((allowance (get-allowance sender operator)))
			(asserts! (>= allowance amount) false)
			(and
				(< allowance max-uint128)
				(map-set allowances {sender: sender, operator: operator} (- allowance amount))
			)
			true
		)
	)
)

(define-public (request-peg-out (amount uint) (sender principal) (destination { version: (buff 1), hashbytes: (buff 32) }))
	(begin
		;; Check if the operator is allowed to request a peg-out for sender for the specified amount.
		(asserts! (is-allowed-operator-and-deduct sender contract-caller amount) err-operator-not-allowed)
		;; Lock the tokens.
		(unwrap! (contract-call? .sbtc-token protocol-lock amount sender) err-token-lock-failed)
		;; Insert the request, returns the peg-out request-id.
		(contract-call? .sbtc-registry insert-peg-out-request amount sender (get-expiry-burn-height) destination 0x)
	)
)

(define-public (request-peg-out-sponsored (amount uint) (sender principal) (destination { version: (buff 1), hashbytes: (buff 32) }) (fee uint))
	(begin
		;; Check if the operator is allowed to request a peg-out for sender for the specified amount.
		(asserts! (is-allowed-operator-and-deduct sender contract-caller amount) err-operator-not-allowed)
		;; Pay the fee.
		(try! (contract-call? .sbtc-token protocol-transfer fee sender (unwrap! tx-sponsor? err-no-sponsor)))
		;; Lock the tokens.
		(unwrap! (contract-call? .sbtc-token protocol-lock amount sender) err-token-lock-failed)
		;; Insert the request, returns the peg-out request-id.
		(contract-call? .sbtc-registry insert-peg-out-request amount sender (get-expiry-burn-height) destination 0x)
	)
)
