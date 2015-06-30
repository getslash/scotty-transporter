.PHONY: deb
deb:
	./deb/pack.sh

.PHONY: clean
clean:
	rm -rf target/release_dir
	rm -rf *.deb
