#!/bin/sh
set -eux

# This script is written to be as POSIX as possible
# so it works fine for all Unix-like operating systems

test_cmd() {
  command -v "$1" >/dev/null
}

# proxy version
photon_new_ver="${1}"
# proxy directory
# eval to resolve '~' into proper user dir
eval photon_dir="'${2}'"

case "${photon_new_ver}" in
  v*)
    photon_new_version=$(echo "${photon_new_ver}" | cut -d'v' -f2)
    photon_new_ver_tag="${photon_new_ver}"
  ;;
  nightly*)
    photon_new_version="${photon_new_ver}"
    photon_new_ver_tag=$(echo ${photon_new_ver} | cut -d '-' -f1)
  ;;
  *)
    printf 'Unknown version\n'
    exit 1
  ;;
esac

if [ -e "${photon_dir}/photon" ]; then
  photon_installed_ver=$("${photon_dir}/photon" --version | cut -d' ' -f2)

  printf '[DEBUG]: Current proxy version: %s\n' "${photon_installed_ver}"
  printf '[DEBUG]: New proxy version: %s\n' "${photon_new_version}"
  if [ "${photon_installed_ver}" = "${photon_new_version}" ]; then
    printf 'Proxy already exists\n'
    exit 0
  else
    printf 'Proxy outdated. Replacing proxy\n'
    rm "${photon_dir}/photon"
  fi
fi

for _cmd in tar gzip uname; do
  if ! test_cmd "${_cmd}"; then
    printf 'Missing required command: %s\n' "${_cmd}"
    exit 1
  fi
done

# Currently only linux/darwin are supported
case $(uname -s) in
  Linux) os_name=linux ;;
  Darwin) os_name=darwin ;;
  *)
    printf '[ERROR] unsupported os\n'
    exit 1
  ;;
esac

# Currently only amd64/arm64 are supported
case $(uname -m) in
  x86_64|amd64|x64) arch_name=x86_64 ;;
  arm64|aarch64) arch_name=aarch64 ;;
  # riscv64) arch_name=riscv64 ;;
  *)
    printf '[ERROR] unsupported arch\n'
    exit 1
  ;;
esac

photon_download_url="https://github.com/lapce/lapce/releases/download/${photon_new_ver_tag}/photon-proxy-${os_name}-${arch_name}.gz"

printf 'Creating "%s"\n' "${photon_dir}"
mkdir -p "${photon_dir}"
cd "${photon_dir}"

if test_cmd 'curl'; then
  # How old curl has these options? we'll find out
  printf 'Downloading using curl\n'
  curl --proto '=https' --tlsv1.2 -LfS -O "${photon_download_url}"
  # curl --proto '=https' --tlsv1.2 -LZfS -o "${tmp_dir}/photon-proxy-${os_name}-${arch_name}.gz" "${photon_download_url}"
elif test_cmd 'wget'; then
  printf 'Downloading using wget\n'
  wget "${photon_download_url}"
else
  printf 'curl/wget not found, failed to download proxy\n'
  exit 1
fi

printf 'Decompressing gzip\n'
gzip -df "${photon_dir}/photon-proxy-${os_name}-${arch_name}.gz"

printf 'Renaming proxy \n'
mv -v "${photon_dir}/photon-proxy-${os_name}-${arch_name}" "${photon_dir}/photon"

printf 'Making it executable\n'
chmod +x "${photon_dir}/photon"

printf 'photon-proxy installed\n'

exit 0
