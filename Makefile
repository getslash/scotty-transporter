pack:
	docker build -t transporter .
	docker run -v $(PWD):/src --rm -ti transporter

clean:
	rm -rf target/release_dir
	rm -rf *.deb
