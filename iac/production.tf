provider "aws" {
  region = "eu-central-1"  # specify your region here
}

resource "aws_instance" "TuCocheDanaBot_Pro" {
  ami                         = "ami-0084a47cc718c111a"
  instance_type               = "t2.micro"
  key_name                    = "TuCocheDana-Alvaro"
  hibernation                 = true

  credit_specification {
    cpu_credits = "standard"
  }

  # Network Interface
#   network_interface {
#     device_index          = 0
#     associate_public_ip_address = true
#     security_groups       = ["sg-0e650299d116e5c7d"]
#   }

  # Block Device Mapping
  root_block_device {
    encrypted           = true
    delete_on_termination = true
    iops                = 3000
    volume_size         = 8
    volume_type         = "gp3"
    throughput          = 125
    //snapshot_id         = "snap-027fa0857ea3f5c8f"
  }

  # Metadata Options
  metadata_options {
    http_endpoint               = "enabled"
    http_put_response_hop_limit = 2
    http_tokens                 = "required"
  }

  # Private DNS Name Options
  private_dns_name_options {
    hostname_type               = "ip-name"
    enable_resource_name_dns_a_record    = true
    enable_resource_name_dns_aaaa_record = false
  }

  # Tags
  tags = {
    Name = "TuCocheDanaBot-Pro"
  }

  count = 1
}
