#!/usr/bin/env python3
import torch
import numpy as np
from torch.nn import Module
import torch.nn.functional as F

width = height = 8

class DemoModule(Module):
    def __init__(self):
        super().__init__()
        self.conv1 = torch.nn.Conv2d(1, 8, kernel_size=(2, 2))
        self.conv2 = torch.nn.Conv2d(8, 16, kernel_size=(2,2))
        self.pool = torch.nn.MaxPool2d((2,2))
        self.flatten = torch.nn.Flatten()
        self.linear1 = torch.nn.Linear(16 * (width-5) * (height-5), 70)
        self.linear2 = torch.nn.Linear(70, 50)
        self.linear3 = torch.nn.Linear(50, 1)

    def forward(self, x):
        x = torch.relu(self.conv1(x))
        x = torch.relu(self.conv2(x))
        x = self.pool(x)
        x = self.flatten(x)
        x = torch.relu(self.linear1(x))
        x = torch.relu(self.linear2(x))
        x = torch.tanh(self.linear3(x))
        return x



a = np.arange(5*8*8)
a.resize((5,1,8,8))
a = torch.tensor(a)
a = a.double()
print(a.size())
net = DemoModule()
net = net.double()
print(net(a))

traced_script_module = torch.jit.script(net)
traced_script_module.save("models/stack4_model.pt")