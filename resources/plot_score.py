#!/usr/bin/env python3

import json
import sys
import matplotlib.pyplot as plt


if len(sys.argv) < 2:
    print("usage: plot_score.py JSON_FILE")
arg = sys.argv[1]
data = json.load(open(arg,"r"))

plt.plot(data)
plt.ylim([-1.1,1.1])
plt.grid()
plt.show()