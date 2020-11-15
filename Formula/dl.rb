class dl < Formula
  desc "A small download utility, written in Rust"
  homepage "https://github.com/fraserdarwent/dl"
  url "https://github.com/MaterializeInc/materialize/archive/v0.5.1.tar.gz"
  
  sha256 "30d83bde5ff421803ebacdd1a36be757c53bd71e3a7bc5468de809175fa58733"
  head "https://github.com/fraserdarwent/dl.git", branch: "main"

  bottle :unneeded

  # bottle do
  #   root_url "https://packages.materialize.io/homebrew"
  #   sha256 "8fc60a92edfd81108eb7abead1b95963eec0c98337296e2d7e91d8a198118685" => :high_sierra
  end

  depends_on "cmake" => :build
  depends_on "rust" => :build

  def install
    # Materialize uses a procedural macro that invokes "git" in order to embed
    # the current SHA in the built binary. The MZ_DEV_BUILD_SHA variable
    # blocks that macro from running at build-time.
    ENV["MZ_DEV_BUILD_SHA"] = STABLE_BUILD_SHA if stable?
    system "cargo", "install", "--locked",
                               "--root", prefix,
                               "--path", "src/materialized"
  end

  # def caveats
  #   <<~EOF
  #     The launchd service will use only one worker thread. For improved
  #     performance, consider manually starting materialized and tuning the
  #     number of worker threads based on your hardware:
  #         materialized --threads=N
  #   EOF
  # end

  plist_options manual: "materialized --threads=1"

  def plist
    <<~EOS
      <?xml version="1.0" encoding="UTF-8"?>
      <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
      <plist version="1.0">
      <dict>
        <key>Label</key>
        <string>#{plist_name}</string>
        <key>ProgramArguments</key>
        <array>
          <string>#{opt_bin}/materialized</string>
          <string>--data-directory=#{var}/materialized</string>
          <string>--threads=1</string>
        </array>
        <key>WorkingDirectory</key>
        <string>#{var}</string>
        <key>RunAtLoad</key>
        <true/>
        <key>KeepAlive</key>
        <true/>
      </dict>
      </plist>
    EOS
  end

  test do
    pid = fork do
      exec bin/"materialized", "-w1"
    end
    sleep 2

    output = shell_output("curl 127.0.0.1:6875")
    assert_includes output, build_sha

    output = shell_output("materialized --version").chomp
    assert_equal output, "materialized v#{version} (#{build_sha})"
  ensure
    Process.kill(9, pid)
    Process.wait(pid)
  end
end
