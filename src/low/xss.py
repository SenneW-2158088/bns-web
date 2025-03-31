import sys

input_file = sys.argv[1]
output_file = sys.argv[2]

with open(input_file, 'r') as file:
    content = file.read()

content = ' '.join(content.split())

with open(output_file, 'w') as file:
    file.write(content)
