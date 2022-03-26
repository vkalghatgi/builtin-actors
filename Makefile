SHELL=/usr/bin/env bash

ORDERED_PACKAGES:=fil_actors_runtime \
                  fil_actor_account \
                  fil_actor_cron \
                  fil_actor_init \
                  fil_actor_market \
                  fil_actor_miner \
                  fil_actor_multisig \
                  fil_actor_paych \
                  fil_actor_power \
                  fil_actor_reward \
                  fil_actor_system \
                  fil_actor_verifreg \
                  fil_builtin_actors_bundle

VERSIONS:=patch minor major alpha beta

BUNDLE_VERSION=$(shell cargo metadata -q --format-version=1 --no-deps | jq -r '.packages[] | select(.name == "fil_builtin_actors_bundle") | .version')

check:
	cargo check --workspace --tests --benches --lib --bins --examples

test: check
	cargo test --workspace

$(addprefix release_,$(VERSIONS)): check_clean check_deps test
	echo "$(ORDERED_PACKAGES)" | xargs -n1 cargo set-version --bump $(patsubst release_%,%,$@) -p
	cargo update --workspace
	git commit -a -m "Release $(BUNDLE_VERSION)"

publish:
	echo "$(ORDERED_PACKAGES)" | xargs -n1 cargo publish -p

check_clean:
	git diff --quiet || $(error Working tree dirty, please commit any changes first)

check_deps:
	which jq >/dev/null 2>&1 || $(error Please install jq)
	which cargo-set-version >/dev/null 2>&1 || $(error Please install cargo-edit: cargo install cargo-edit)


.PHONY: $(addprefix release_,$(VERSIONS)) check_clean check_deps test publish
