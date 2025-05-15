#!/bin/sh

function mount_shared_vm_fs () {
    local VM_FS=$1
    local VM_PATH=$2

    if [ -n "$(mount | grep $VM_PATH)" ]; then
        return 1
    fi

    mkdir -p $VM_PATH
    mount -t 9p -o trans=virtio $VM_FS "$VM_PATH" -oversion=9p2000.L
    return 0
}

mount_shared_vm_fs $1 $2