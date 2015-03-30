# -*- mode: ruby -*-
# vi: set ft=ruby :

VAGRANTFILE_API_VERSION = "2"

Vagrant.configure(VAGRANTFILE_API_VERSION) do |config|
  config.vm.define "ubuntu" do |ubuntu|
    ubuntu.vm.box = "ubuntu/trusty64"
  end

  config.vm.provision "ansible" do |ansible|
      ansible.groups = {
        "transporters" => ["ubuntu"],
      }
      ansible.playbook = "ansible/site.yml"
      ansible.sudo = true
   end
end
