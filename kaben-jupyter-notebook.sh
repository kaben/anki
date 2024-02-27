#!/bin/bash

source out/pyenv/bin/activate
EXISTING_CONNECTION_FILE=kernel-anki.json ./out/pyenv/bin/jupyter lab --KernelProvisionerFactory.default_provisioner_name=existing-provisioner
