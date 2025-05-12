.PHONY: all cgroup_v1 cgroup_v2 clean

all: build/initrd.gz

build/initrd.gz: cgroup_v1 cgroup_v2
	rm -f ./build/mnt/root/test_suite
	ln -s /root/test_suite_v2 ./build/mnt/root/test_suite
	cd ./build/mnt; find . | cpio -o -H newc | gzip > ../initrd.gz

build/mnt/.keep:
	mkdir -p ./build/mnt
	cp -ur ./mnt ./build/

build/.keep: build/mnt/.keep
	mkdir -p ./build
	touch ./build/.keep

cgroup_v2: build/.keep
	mkdir -p ./build/test_suite/v2
	cd src; cargo build --release --features cgroup_v2
	cargo install --path ./src --root ./build/test_suite/v2 --no-track --frozen --features cgroup_v2
	mkdir -p ./build/mnt/root/test_suite_v2
	cp -r ./build/test_suite/v2/bin/* ./build/mnt/root/test_suite_v2/
	rm -rf ./build/test_suite/v2

cgroup_v1: build/.keep
	mkdir -p ./build/test_suite/v1
	cd src; cargo build --release
	cargo install --path ./src --root ./build/test_suite/v1 --no-track --frozen
	mkdir -p ./build/mnt/root/test_suite_v1
	cp -r ./build/test_suite/v1/bin/* ./build/mnt/root/test_suite_v1/
	rm -rf ./build/test_suite/v1

clean:
	rm -rf ./build
