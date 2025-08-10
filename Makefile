.PHONY: all core install cgroup cgroup_v1 cgroup_v2 periodic busybox tasksets clean

BUILD?=./build

all: $(BUILD)/core.gz $(BUILD)/install.tar.gz

core: $(BUILD)/core.gz
install: $(BUILD)/install.tar.gz

$(BUILD)/core.gz: $(BUILD)/initrd.gz $(BUILD)/initrd-busybox.gz $(BUILD)/initrd-periodic.gz $(BUILD)/initrd-tasksets.gz $(BUILD)/initrd-scripts.gz
	rm -f $(BUILD)/core.gz
	touch $(BUILD)/core.gz
	cat $(BUILD)/initrd.gz >> $(BUILD)/core.gz
	cat $(BUILD)/initrd-busybox.gz >> $(BUILD)/core.gz
	cat $(BUILD)/initrd-periodic.gz >> $(BUILD)/core.gz
	cat $(BUILD)/initrd-tasksets.gz >> $(BUILD)/core.gz
	cat $(BUILD)/initrd-scripts.gz >> $(BUILD)/core.gz


$(BUILD)/install.tar.gz: $(BUILD)/install-periodic.tar.gz $(BUILD)/install-test.tar.gz $(BUILD)/install-tasksets.tar.gz $(BUILD)/install-scripts.tar.gz
	touch $(BUILD)/install.tar.gz
	cat $(BUILD)/install-periodic.tar.gz >> $(BUILD)/install.tar.gz
	cat $(BUILD)/install-test.tar.gz >> $(BUILD)/install.tar.gz
	cat $(BUILD)/install-tasksets.tar.gz >> $(BUILD)/install.tar.gz
	cat $(BUILD)/install-scripts.tar.gz >> $(BUILD)/install.tar.gz
	mkdir -p $(BUILD)/install
	cd $(BUILD)/install && tar -ixvf ../install.tar.gz
	cd $(BUILD)/install && tar -czvf ../install.tar.gz .
	rm -r $(BUILD)/install

# periodic task runner
periodic: $(BUILD)/initrd-periodic.gz

$(BUILD)/PeriodicTask/.keep: $(BUILD)/.keep
	if [ ! -d $(BUILD)/PeriodicTask ]; then\
		git init $(BUILD)/PeriodicTask;\
		git -C $(BUILD)/PeriodicTask fetch --depth=1 https://gitlab.retis.santannapisa.it/l.abeni/PeriodicTask.git 8b1839d2c2207cbb7e80f25e9d6773bbeab6630e;\
		git -C $(BUILD)/PeriodicTask checkout FETCH_HEAD;\
		sed -i '18 c#define MAX_TH 50' $(BUILD)/PeriodicTask/periodic_thread.c;\
	fi
	make -C $(BUILD)/PeriodicTask periodic_task
	make -C $(BUILD)/PeriodicTask periodic_thread
	touch $(BUILD)/PeriodicTask/.keep

$(BUILD)/initrd-periodic.gz: $(BUILD)/PeriodicTask/.keep
	mkdir -p $(BUILD)/periodic-task-initrd/bin
	cp $(BUILD)/PeriodicTask/periodic_task $(BUILD)/periodic-task-initrd/bin/periodic_task
	cp $(BUILD)/PeriodicTask/periodic_thread $(BUILD)/periodic-task-initrd/bin/periodic_thread
	cd $(BUILD)/periodic-task-initrd && find . | cpio -o -H newc | gzip > ../initrd-periodic.gz

$(BUILD)/install-periodic.tar.gz: $(BUILD)/PeriodicTask/.keep
	mkdir -p $(BUILD)/periodic-task-install/bin
	cp $(BUILD)/PeriodicTask/periodic_task $(BUILD)/periodic-task-install/bin/periodic_task
	cp $(BUILD)/PeriodicTask/periodic_thread $(BUILD)/periodic-task-install/bin/periodic_thread
	cd $(BUILD)/periodic-task-install && tar -czvf ../install-periodic.tar.gz bin/

# busybox
busybox: $(BUILD)/initrd-busybox.gz

$(BUILD)/initrd-busybox.gz: $(BUILD)/.keep
	mkdir -p $(BUILD)/busybox
	# get busybox builder and update the config
	if [ ! -d $(BUILD)/BuildCore ]; then\
		git init $(BUILD)/BuildCore;\
		git -C $(BUILD)/BuildCore fetch --depth=1 https://gitlab.retis.santannapisa.it/l.abeni/BuildCore.git 715962453dc89fb694f1193278d9f45304f03741;\
		git -C $(BUILD)/BuildCore checkout FETCH_HEAD;\
		sed -i '967 cCONFIG_TC=n' $(BUILD)/BuildCore/Configs/config-busybox-3;\
		sed -i '11 cSUDOVER=1.9.17p2' $(BUILD)/BuildCore/buildcore.sh;\
	fi
	cd $(BUILD)/busybox && sh $(BUILD)/BuildCore/buildcore.sh $(BUILD)/initrd-busybox.gz

# tasksets
tasksets: $(BUILD)/initrd-tasksets.gz $(BUILD)/install-tasksets.tar.gz

$(BUILD)/tasksets/.keep: $(BUILD)/.keep
	mkdir -p $(BUILD)/tasksets
	touch $(BUILD)/tasksets/.keep
	# get CARTS
	if [ ! -d $(BUILD)/SchedTest ]; then\
		tar -C $(BUILD) -xf $(shell pwd)/sched_test.tgz;\
	fi
	cd taskset_gen && BUILD=$(BUILD) python -B taskgen.py -o $(BUILD)/tasksets/root/tasksets

$(BUILD)/initrd-tasksets.gz: $(BUILD)/tasksets/.keep
	cd $(BUILD)/tasksets && find . | cpio -o -H newc | gzip > ../initrd-tasksets.gz

$(BUILD)/install-tasksets.tar.gz: $(BUILD)/tasksets/.keep
	cd $(BUILD)/tasksets/root && tar -czvf ../../install-tasksets.tar.gz .

# test software
cgroup: $(BUILD)/initrd.gz $(BUILD)/install-test.tar.gz

$(BUILD)/initrd.gz: cgroup_v1 cgroup_v2
	rm -f $(BUILD)/mnt/root/test_suite
	ln -s /root/test_suite_v2 $(BUILD)/mnt/root/test_suite
	cd $(BUILD)/mnt && find . | cpio -o -H newc | gzip > ../initrd.gz

$(BUILD)/install-test.tar.gz: cgroup_v1 cgroup_v2
	cd $(BUILD)/mnt/root/ && tar -czvf ../../install-test.tar.gz test_suite_v2/ test_suite_v1/

cgroup_v2: $(BUILD)/mnt/.keep $(BUILD)/.keep
	mkdir -p $(BUILD)/test_suite/v2
	cd test_suite_rs && RUSTFLAGS='-C target-feature=+crt-static' CARGO_HOME='$(BUILD)/rust/cargo' CARGO_TARGET_DIR='$(BUILD)/rust/target' \
		cargo build --release --features cgroup_v2 --target x86_64-unknown-linux-gnu
	RUSTFLAGS='-C target-feature=+crt-static' CARGO_HOME='$(BUILD)/rust/cargo' CARGO_TARGET_DIR='$(BUILD)/rust/target' \
		cargo install --path ./test_suite_rs --root $(BUILD)/test_suite/v2 \
		--no-track --frozen --features cgroup_v2 --target x86_64-unknown-linux-gnu
	mkdir -p $(BUILD)/mnt/root/test_suite_v2
	cp -r $(BUILD)/test_suite/v2/bin/* $(BUILD)/mnt/root/test_suite_v2/
	rm -rf $(BUILD)/test_suite/v2

cgroup_v1: $(BUILD)/mnt/.keep $(BUILD)/.keep
	mkdir -p $(BUILD)/test_suite/v1
	cd test_suite_rs && RUSTFLAGS='-C target-feature=+crt-static' CARGO_HOME='$(BUILD)/rust/cargo' CARGO_TARGET_DIR='$(BUILD)/rust/target' \
		cargo build --release --target x86_64-unknown-linux-gnu
	RUSTFLAGS='-C target-feature=+crt-static' CARGO_HOME='$(BUILD)/rust/cargo' CARGO_TARGET_DIR='$(BUILD)/rust/target' \
		cargo install --path ./test_suite_rs --root $(BUILD)/test_suite/v1 \
		--no-track --frozen --target x86_64-unknown-linux-gnu
	mkdir -p $(BUILD)/mnt/root/test_suite_v1
	cp -r $(BUILD)/test_suite/v1/bin/* $(BUILD)/mnt/root/test_suite_v1/
	rm -rf $(BUILD)/test_suite/v1

# extra scripts
$(BUILD)/initrd-scripts.gz:
	cd scripts && find . | cpio -o -H newc | gzip > $(BUILD)/initrd-scripts.gz

$(BUILD)/install-scripts.tar.gz:
	cd scripts && tar -czvf $(BUILD)/install-scripts.tar.gz .

# generic
$(BUILD)/mnt/.keep:
	mkdir -p $(BUILD)/mnt
	cp -ur ./mnt $(BUILD)/

$(BUILD)/.keep:
	mkdir -p $(BUILD)
	touch $(BUILD)/.keep

clean:
	rm -rf $(BUILD)
	cd test_suite_rs && cargo clean
