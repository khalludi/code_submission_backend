# collections imports
from collections import *
# Receive inputs from command line
import sys
numCourses = int(sys.argv[1])
strPreReq = sys.argv[2]

# Use regex to convert prereqs string to 2D array
import re
reg2 = re.compile(r'\[(\d+)\,(\d+)\]')
prereqs = []
for a, b in reg2.findall(strPreReq):
    prereqs.append([int(a), int(b)])

# Call Solution and print output
s = Solution()
print(s.solve(numCourses, prereqs))