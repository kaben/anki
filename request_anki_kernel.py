import argparse
import sys

import requests


def get_arg_parser():
  parser = argparse.ArgumentParser(
    prog = 'request_anki_kernel',
  )
  parser.add_argument('--url-base')
  parser.add_argument('--connection-file')
  return parser

def request_anki_kernel(args):
  payload = dict(connection_file = args.connection_file)
  url = f"{args.url_base}/start_jupyter_kernel"
  response = requests.post(url, params = payload)
  print(response.text)

if __name__ == "__main__":
  parser = get_arg_parser()
  args = parser.parse_args()
  request_anki_kernel(args)
  print(args)