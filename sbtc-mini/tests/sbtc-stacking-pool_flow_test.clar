(define-constant mock-pox-reward-wallet-1 { version: 0x06, hashbytes: 0x0011223344556699001122334455669900112233445566990011223344556699 })

;; cycle windows
(define-constant disbursement 0x00)
(define-constant registration 0x01)
(define-constant voting 0x02)
(define-constant transfer 0x03)
(define-constant penalty 0x04)
(define-constant bad-peg-state 0x05)

;; @poxadmin .sbtc-stacking-pool
;; @caller wallet_1
(define-public (test-sign-pre-register)
	(begin
		(let
			((registration-result
				(contract-call? .sbtc-stacking-pool signer-pre-register u1000 mock-pox-reward-wallet-1)))
			(asserts! (is-ok registration-result) registration-result))
			(ok true)))