.PHONY: all cgroup_v1 cgroup_v2 tasksets clean

all: build/core.gz

build/core.gz: build/initrd.gz build/busybox.gz build/periodic.gz build/tasksets.gz
	rm -f build/core.gz
	touch build/core.gz
	cat ./build/busybox.gz >> ./build/core.gz
	cat ./build/initrd.gz >> ./build/core.gz
	cat ./build/periodic.gz >> ./build/core.gz
	cat ./build/tasksets.gz >> ./build/core.gz

build/periodic.gz: build/.keep
	mkdir -p ./build/periodic-task/bin
	if [ ! -d ./build/PeriodicTask ]; then\
		git init ./build/PeriodicTask;\
		git -C ./build/PeriodicTask fetch --depth=1 https://gitlab.retis.santannapisa.it/l.abeni/PeriodicTask.git 8b1839d2c2207cbb7e80f25e9d6773bbeab6630e;\
		git -C ./build/PeriodicTask checkout FETCH_HEAD;\
	fi
	make -C ./build/PeriodicTask periodic_task
	make -C ./build/PeriodicTask periodic_thread
	cp ./build/PeriodicTask/periodic_task ./build/periodic-task/bin/periodic_task
	cp ./build/PeriodicTask/periodic_thread ./build/periodic-task/bin/periodic_thread
	cd ./build/periodic-task; find . | cpio -o -H newc | gzip > ../periodic.gz

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

tasksets: build/tasksets.gz

build/tasksets.gz: build/.keep
	rm -rf ./build/tasksets/root/tasksets
	# get CARTS (?)
	cd taskset_gen; python -B taskgen.py
	cd ./build/tasksets; find . | cpio -o -H newc | gzip > ../tasksets.gz

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
	cd test_suite_rs; RUSTFLAGS='-C target-feature=+crt-static' \
		cargo build --release --features cgroup_v2 --target x86_64-unknown-linux-gnu
	RUSTFLAGS='-C target-feature=+crt-static' \
		cargo install --path ./test_suite_rs --root ./build/test_suite/v2 \
		--no-track --frozen --features cgroup_v2 --target x86_64-unknown-linux-gnu
	mkdir -p ./build/mnt/root/test_suite_v2
	cp -r ./build/test_suite/v2/bin/* ./build/mnt/root/test_suite_v2/
	rm -rf ./build/test_suite/v2

cgroup_v1: build/mnt/.keep build/.keep
	mkdir -p ./build/test_suite/v1
	cd test_suite_rs; RUSTFLAGS='-C target-feature=+crt-static' \
		cargo build --release --target x86_64-unknown-linux-gnu
	RUSTFLAGS='-C target-feature=+crt-static' \
		cargo install --path ./test_suite_rs --root ./build/test_suite/v1 \
		--no-track --frozen --target x86_64-unknown-linux-gnu
	mkdir -p ./build/mnt/root/test_suite_v1
	cp -r ./build/test_suite/v1/bin/* ./build/mnt/root/test_suite_v1/
	rm -rf ./build/test_suite/v1

clean:
	rm -rf ./build
