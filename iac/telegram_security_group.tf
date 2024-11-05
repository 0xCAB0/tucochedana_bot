resource "aws_security_group" "TuCocheDanaBot_Pro_SG" {
  name        = "TuCocheDanaBot-Pro-SG"
  description = "Security group for TuCocheDanaBot-Pro allowing specific IP ranges on certain ports"
  vpc_id      = "vpc-xxxxxxxx"  # Replace with your actual VPC ID

  # Inbound rules for IP range 149.154.160.0/20
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["149.154.160.0/20"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["149.154.160.0/20"]
  }

  ingress {
    from_port   = 88
    to_port     = 88
    protocol    = "tcp"
    cidr_blocks = ["149.154.160.0/20"]
  }

  ingress {
    from_port   = 8443
    to_port     = 8443
    protocol    = "tcp"
    cidr_blocks = ["149.154.160.0/20"]
  }

  # Inbound rules for IP range 91.108.4.0/22
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["91.108.4.0/22"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["91.108.4.0/22"]
  }

  ingress {
    from_port   = 88
    to_port     = 88
    protocol    = "tcp"
    cidr_blocks = ["91.108.4.0/22"]
  }

  ingress {
    from_port   = 8443
    to_port     = 8443
    protocol    = "tcp"
    cidr_blocks = ["91.108.4.0/22"]
  }

  # Egress rule to allow all outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "Telegram-SG"
  }
}