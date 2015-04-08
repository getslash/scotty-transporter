pack:
	mkdir -p target/deb
	docker build -t transporter .
	docker run -v $(PWD):/src -v $(PWD)/target/deb:/src/target --rm -ti transporter

clean:
	rm -rf target/release_dir
	rm -rf *.deb
