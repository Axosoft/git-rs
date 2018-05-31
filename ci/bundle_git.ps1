[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$url = "https://github.com/desktop/dugite-native/releases/download/v2.17.1/dugite-native-v2.17.1-win32.tar.gz"
Invoke-WebRequest -Uri $url -OutFile vendor.tar.gz

7z x vendor.tar.gz
7z x vendor.tar -ovendor
7z a -ttar x86_64-pc-windows-msvc.tar vendor git_server.exe
7z a -tgzip x86_64-pc-windows-msvc.tar.gz x86_64-pc-windows-msvc.tar
