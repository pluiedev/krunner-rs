build:
	nix build -L

run: build
	./result/bin/krunner_nix
