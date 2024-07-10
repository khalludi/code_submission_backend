from collections import *

class Grader:
    def solve(self, numCourses, prerequisites):
        indegrees = [0 for _ in range(numCourses)]
        adjList = defaultdict(list)
        for post, pre in prerequisites:
          adjList[pre].append(post)
          indegrees[post] += 1

        q = deque()
        for i in range(numCourses):
          if indegrees[i] == 0:
            q.append(i)

        while q:
          curr = q.pop()

          for c in adjList[curr]:
            indegrees[c] -= 1
            if indegrees[c] == 0:
              q.append(c)

        for c in indegrees:
          if c > 0:
            return False

        return True

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

# Call grader and print output
g = Grader()
print(g.solve(numCourses, prereqs))