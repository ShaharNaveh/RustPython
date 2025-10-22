import io

import _tokenize


def meep():
    return 123


'''
b = io.StringIO(
    """
for i in range(10):



  pass
""".lstrip()
)
'''

b = io.StringIO(
    """
for i in range(10):



    pass
""".lstrip()
)


# b = io.StringIO(None)

it = _tokenize.TokenizerIter(b.readline, extra_tokens=False)
# it = _tokenize.TokenizerIter(meep, extra_tokens=False)
for i in it:
    print(i)

exit()

print()
print()
print()


b = io.StringIO(
    """
for i in range(10):
""".lstrip()
)

# b = io.StringIO(None)

it = _tokenize.TokenizerIter(b.readline, extra_tokens=True)
# it = _tokenize.TokenizerIter(meep, extra_tokens=False)
for i in it:
    print(i)
