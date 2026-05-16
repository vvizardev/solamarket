PKG := smart-contract
CRATE := $(subst -,_,$(PKG))
PROJECT_NAME := $(CRATE)
DEPLOY := ./target/deploy
PROGRAM_KEY := $(DEPLOY)/$(PROJECT_NAME)-keypair.json
FEE_PAYER := ./id.json
SO := $(DEPLOY)/$(CRATE).so

.PHONY: rotate addr build deploy migrate update

rotate:
	rm -f $(PROGRAM_KEY)
	solana-keygen new -o $(PROGRAM_KEY)

addr:
	solana address -k $(PROGRAM_KEY)

build:
	rm -f $(SO)
	cargo build-sbf

deploy:
	solana program deploy $(SO) --program-id $(PROGRAM_KEY) --fee-payer $(FEE_PAYER)

migrate:
	solana program migrate "$$(solana address -k $(PROGRAM_KEY))"

update: build deploy
