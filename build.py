from __future__ import print_function

import os
import sys
import zipfile
import subprocess
import shutil
import platform
import tempfile
import argparse
import json
import tarfile
import time
import re

try:
  from urllib.request import urlopen, urlretrieve, Request
  from urllib.error import HTTPError
except ImportError:
  from urllib import urlretrieve
  from urllib2 import urlopen, Request

def main():
  parser = argparse.ArgumentParser('build tool for project-a')
  parser.add_argument('--module', action='append', choices=['event', 'render'])
  parser.add_argument('--out-dir')
  args = parser.parse_args()

  rustup_cmd = 'rustup'

  try:
    process = subprocess.Popen([rustup_cmd, '-V'], stdout=subprocess.PIPE, cwd=os.path.dirname(os.path.abspath(__file__)))
    out, _ = process.communicate()
  except:
    install_path = os.path.dirname(os.path.abspath(__file__)) + "/.rust-install/"
    rustup_cmd = install_path + '.cargo/bin/rustup.exe'

    os.environ["RUSTUP_HOME"] = install_path + ".rustup"
    os.environ["CARGO_HOME"] = install_path + ".cargo"

    if not os.path.exists(install_path):
      path = os.path.join(tempfile.mkdtemp(), 'rustup-init.exe')
      urlretrieve('https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe', path)

      process = subprocess.Popen([path, '-y', '--no-modify-path', '--profile', 'minimal', '--default-toolchain', 'none'])
      process.wait()

  process = subprocess.Popen([rustup_cmd, 'which', 'cargo'], stdout=subprocess.PIPE, cwd=os.path.dirname(os.path.abspath(__file__)))
  out, _ = process.communicate()

  cargo_cmd = out.decode('utf-8').strip()

  build_cmd = [cargo_cmd, '--color', 'always', 'build', '--lib', '-Zunstable-options', '--out-dir={}'.format(os.path.abspath(args.out_dir)), '--features={}'.format(','.join(map(lambda x: 'c-{}'.format(x), args.module)))]

  process = subprocess.Popen(build_cmd)
  process.wait()

  f = open('{}/{}'.format(os.path.dirname(os.path.abspath(__file__)), 'include/generated.h'), 'w')
  for module in args.module:
    f.write('#define MODULE_{}\n'.format(module.upper()))
  f.close()

if __name__ == '__main__':
  sys.exit(main())