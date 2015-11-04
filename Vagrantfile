# -*- mode: ruby -*-
# vi: set ft=ruby :

VAGRANTFILE_API_VERSION = "2"

Vagrant.configure(VAGRANTFILE_API_VERSION) do |config|
  config.vm.define "debian" do |debian|
    debian.vm.box = "debian/jessie64"
  end

  config.vm.provision "ansible" do |ansible|
      ansible.groups = {
        "transporters" => ["debian"],
      }
      ansible.playbook = "ansible/site.yml"
      ansible.sudo = true
   end
end
