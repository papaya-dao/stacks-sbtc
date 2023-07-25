(define-constant mock-pox-reward-wallet-1 { version: 0x06, hashbytes: 0x0011223344556699001122334455669900112233445566990011223344556699 })

;; @name user can pre-register
;; @caller wallet_1
(define-public (test-sign-pre-register)
	(begin
        ;; @continue
        (unwrap! (contract-call? .pox-3 mock-set-stx-account 'ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5 {locked: u1000000000, unlock-height: u100, unlocked: u100000000000}) (err u111))
        ;; @continue
        (unwrap! (contract-call? .pox-3 allow-contract-caller 'ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.sbtc-stacking-pool none) (err u1111))
        ;; @mine-blocks-before 5
		(check-sign-pre-register))
)

(define-public (check-sign-pre-register)
    (let
        ((registration-result
				(contract-call? .sbtc-stacking-pool signer-pre-register u10000000000000 mock-pox-reward-wallet-1)))
			(asserts! (is-ok registration-result) registration-result)
			(ok true)))