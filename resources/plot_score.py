#!/usr/bin/env python3

import numpy as np
import json
import sys
import argparse
import matplotlib.pyplot as plt

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--json_file", action='store', type=str, help="json file of scores to be plotted")    
    parser.add_argument('--movmean', action='store', type=int, help="Takes moving average over specified number of points")
    args = parser.parse_args()
    if args.json_file is not None:
        # Gets data to plot from file.
        data = json.load(open(args.json_file,"r"))
    else:
        # Gets data to plot from stdin.
        data = json.loads(sys.stdin.read())
    
    if args.movmean is not None:
        n = args.movmean
        data = np.convolve(data, np.ones(n), 'valid')/n
    plt.plot(data)
    plt.ylim([-1.2,1.2])
    plt.grid()
    plt.show()


if __name__=="__main__":
    main()