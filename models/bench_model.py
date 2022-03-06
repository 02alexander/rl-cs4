#!/usr/bin/env python3
import torch
import numpy as np
from torch.nn import Module
import torch.nn.functional as F

width = 7
height = 6

class DemoModule(Module):
    def __init__(self):
        super().__init__()
        self.conv1 = torch.nn.Conv2d(1, 4, kernel_size=(3, 3), padding=(1,1))
        #self.pool = torch.nn.MaxPool2d((2,2))
        self.flatten = torch.nn.Flatten()
        self.linear1 = torch.nn.Linear(4 * width * height, 20)
        self.linear2 = torch.nn.Linear(20, 1)

    def forward(self, x):
        x = torch.relu(self.conv1(x))
        #x = self.pool(x)
        x = self.flatten(x)
        x = torch.relu(self.linear1(x))
        x = self.linear2(x)
        return x


a = np.arange(5*6*7)
a.resize((5,1,6,7))
a = torch.tensor(a)
a = a.double()
print(a.size())
net = DemoModule()
net = net.double()
print(net(a))

traced_script_module = torch.jit.script(net)
traced_script_module.save("benches/bench_model.pt")