# Documentation: https://docs.brew.sh/Formula-Cookbook
#                https://rubydoc.brew.sh/Formula
# PLEASE REMOVE ALL GENERATED COMMENTS BEFORE SUBMITTING YOUR PULL REQUEST!
class Dl < Formula
  desc "A small download utility"
  homepage "https://github.com/fraserdarwent/dl"
  url "https://github.com/fraserdarwent/dl/releases/download/0.0.4/dl-0.0.4-darwin-amd64.zip"
  sha256 "1f83fce2c2534ef4fe9bc086c036d66e347cd4e0e6dcdd0ef527b8c1b9dcbc4d"
  license "MIT"

  def install
    bin.install "dl"
  end
end
