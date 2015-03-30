release:
	cargo build --release

release_dir: release
	mkdir -p target/release_dir/usr/bin
	cp target/release/transporter target/release_dir/usr/bin/transporter

deb: release_dir
	fpm -s dir -t deb -n transporter -v `grep version Cargo.toml | awk '{print $$3}' | sed 's/"//g'` -C target/release_dir .

clean:
	rm -rf target/release_dir
	rm -rf *.deb