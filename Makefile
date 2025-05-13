.PHONY: all cgroup_v1 cgroup_v2 clean

all: build/core.gz

build/core.gz: build/initrd.gz build/busybox.gz
	rm -f build/core.gz
	touch build/core.gz
	cat ./build/busybox.gz >> ./build/core.gz
	cat ./build/initrd.gz >> ./build/core.gz

build/busybox.gz: build/.keep
	mkdir -p ./build/busybox
	# get busybox builder and update the config
	if [ ! -d ./build/BuildCore ]; then\
		git init ./build/BuildCore;\
		git -C ./build/BuildCore fetch --depth=1 https://gitlab.retis.santannapisa.it/l.abeni/BuildCore.git 715962453dc89fb694f1193278d9f45304f03741;\
		git -C ./build/BuildCore checkout FETCH_HEAD;\
		sed -i '967 cCONFIG_TC=n' ./build/BuildCore/Configs/config-busybox-3;\
	fi
	cd ./build/busybox; sh $(shell pwd)/build/BuildCore/buildcore.sh $(shell pwd)/build/busybox.gz

build/initrd.gz: cgroup_v1 cgroup_v2
	rm -f ./build/mnt/root/test_suite
	ln -s /root/test_suite_v2 ./build/mnt/root/test_suite
	cd ./build/mnt; find . | cpio -o -H newc | gzip > ../initrd.gz

build/mnt/.keep:
	mkdir -p ./build/mnt
	cp -ur ./mnt ./build/

build/.keep:
	mkdir -p ./build
	touch ./build/.keep

cgroup_v2: build/mnt/.keep build/.keep
	mkdir -p ./build/test_suite/v2
	cd src; cargo build --release --features cgroup_v2
	cargo install --path ./src --root ./build/test_suite/v2 --no-track --frozen --features cgroup_v2
	mkdir -p ./build/mnt/root/test_suite_v2
	cp -r ./build/test_suite/v2/bin/* ./build/mnt/root/test_suite_v2/
	rm -rf ./build/test_suite/v2

cgroup_v1: build/mnt/.keep build/.keep
	mkdir -p ./build/test_suite/v1
	cd src; cargo build --release
	cargo install --path ./src --root ./build/test_suite/v1 --no-track --frozen
	mkdir -p ./build/mnt/root/test_suite_v1
	cp -r ./build/test_suite/v1/bin/* ./build/mnt/root/test_suite_v1/
	rm -rf ./build/test_suite/v1

clean:
	rm -rf ./build
