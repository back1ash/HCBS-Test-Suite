.PHONY: all core install cgroup cgroup_v1 cgroup_v2 periodic busybox tasksets clean

all: build/core.gz build/install.tar.gz

core: build/core.gz
install: build/install.tar.gz

build/core.gz: build/initrd.gz build/initrd-busybox.gz build/initrd-periodic.gz build/initrd-tasksets.gz build/initrd-scripts.gz
	rm -f build/core.gz
	touch build/core.gz
	cat ./build/initrd.gz >> ./build/core.gz
	cat ./build/initrd-busybox.gz >> ./build/core.gz
	cat ./build/initrd-periodic.gz >> ./build/core.gz
	cat ./build/initrd-tasksets.gz >> ./build/core.gz
	cat ./build/initrd-scripts.gz >> ./build/core.gz


build/install.tar.gz: build/install-periodic.tar.gz build/install-test.tar.gz build/install-tasksets.tar.gz build/install-scripts.tar.gz
	touch build/install.tar.gz
	cat build/install-periodic.tar.gz >> build/install.tar.gz
	cat build/install-test.tar.gz >> build/install.tar.gz
	cat build/install-tasksets.tar.gz >> build/install.tar.gz
	cat build/install-scripts.tar.gz >> build/install.tar.gz
	mkdir -p build/install
	cd build/install; tar -ixvf ../install.tar.gz
	cd build/install; tar -czvf ../install.tar.gz .
	rm -r build/install

# periodic task runner
periodic: build/initrd-periodic.gz

build/PeriodicTask/.keep: build/.keep
	if [ ! -d ./build/PeriodicTask ]; then\
		git init ./build/PeriodicTask;\
		git -C ./build/PeriodicTask fetch --depth=1 https://gitlab.retis.santannapisa.it/l.abeni/PeriodicTask.git 8b1839d2c2207cbb7e80f25e9d6773bbeab6630e;\
		git -C ./build/PeriodicTask checkout FETCH_HEAD;\
	fi
	make -C ./build/PeriodicTask periodic_task
	make -C ./build/PeriodicTask periodic_thread
	touch build/PeriodicTask/.keep

build/initrd-periodic.gz: build/PeriodicTask/.keep
	mkdir -p ./build/periodic-task-initrd/bin
	cp ./build/PeriodicTask/periodic_task ./build/periodic-task-initrd/bin/periodic_task
	cp ./build/PeriodicTask/periodic_thread ./build/periodic-task-initrd/bin/periodic_thread
	cd ./build/periodic-task-initrd; find . | cpio -o -H newc | gzip > ../initrd-periodic.gz

build/install-periodic.tar.gz: build/PeriodicTask/.keep
	mkdir -p ./build/periodic-task-install/bin
	cp ./build/PeriodicTask/periodic_task ./build/periodic-task-install/bin/periodic_task
	cp ./build/PeriodicTask/periodic_thread ./build/periodic-task-install/bin/periodic_thread
	cd build/periodic-task-install; tar -czvf ../install-periodic.tar.gz bin/

# busybox
busybox: build/initrd-busybox.gz

build/initrd-busybox.gz: build/.keep
	mkdir -p ./build/busybox
	# get busybox builder and update the config
	if [ ! -d ./build/BuildCore ]; then\
		git init ./build/BuildCore;\
		git -C ./build/BuildCore fetch --depth=1 https://gitlab.retis.santannapisa.it/l.abeni/BuildCore.git 715962453dc89fb694f1193278d9f45304f03741;\
		git -C ./build/BuildCore checkout FETCH_HEAD;\
		sed -i '967 cCONFIG_TC=n' ./build/BuildCore/Configs/config-busybox-3;\
	fi
	cd ./build/busybox; sh $(shell pwd)/build/BuildCore/buildcore.sh $(shell pwd)/build/initrd-busybox.gz

# tasksets
tasksets: build/initrd-tasksets.gz build/install-tasksets.tar.gz

build/tasksets/.keep: build/.keep
	mkdir -p build/tasksets
	touch build/tasksets/.keep
	# get CARTS (?)	
	cd taskset_gen; python -B taskgen.py -o ../build/tasksets/root/tasksets
	# cd taskset_gen; python -B taskgen.py -o ../build/tasksets/root/tasksets_6cpu -T 6 -U 4 -p 50 -P 300 -t 1 -R 1575

build/initrd-tasksets.gz: build/tasksets/.keep
	cd ./build/tasksets; find . | cpio -o -H newc | gzip > ../initrd-tasksets.gz

build/install-tasksets.tar.gz: build/tasksets/.keep
	cd build/tasksets/root; tar -czvf ../../install-tasksets.tar.gz .

# test software
cgroup: build/initrd.gz build/install-test.tar.gz

build/initrd.gz: cgroup_v1 cgroup_v2
	rm -f ./build/mnt/root/test_suite
	ln -s /root/test_suite_v2 ./build/mnt/root/test_suite
	cd ./build/mnt; find . | cpio -o -H newc | gzip > ../initrd.gz

build/install-test.tar.gz: cgroup_v1 cgroup_v2
	cd build/mnt/root/; tar -czvf ../../install-test.tar.gz test_suite_v2/ test_suite_v1/

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

# extra scripts
build/initrd-scripts.gz:
	cd scripts; find . | cpio -o -H newc | gzip > ../build/initrd-scripts.gz

build/install-scripts.tar.gz:
	cd scripts; tar -czvf ../build/install-scripts.tar.gz .

# generic
build/mnt/.keep:
	mkdir -p ./build/mnt
	cp -ur ./mnt ./build/

build/.keep:
	mkdir -p ./build
	touch ./build/.keep

clean:
	rm -rf ./build
	cd test_suite_rs; cargo clean
