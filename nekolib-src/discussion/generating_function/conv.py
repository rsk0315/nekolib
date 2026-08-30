from sympy import collect, symbols

n = 5
a = symbols(f"a(0:{n + 1})")
b = symbols(f"b(0:{n + 1})")
x = symbols("x")
a_rev = a[::-1]

# c = 0
# for i in range(n + 1):
#     for j in range(i + 1):
#         c += a[j] * b[i - j] * x**i

# print(collect(c, x))

c = 0
for i in range(n + 1):
    for j in range(i, n + 1):
        c += a[j] * b[j - i] * x**i

print(collect(c, x))

c = 0
for i in range(n + 1):
    for j in range(i, n + 1):
        c += a[n - j + i] * b[n - j] * x**i

print(collect(c, x))

c = 0
for n_minus_i in range(n + 1):
    i = n - n_minus_i
    for n_minus_j in range(i + 1):
        j = n - n_minus_j
        c += a[n - j + n - i] * b[n - j] * x ** (n - i)

print(collect(c, x))

c = 0
for i in range(n + 1):
    for n_minus_j in range(n - i + 1):
        j = n - n_minus_j
        c += a_rev[(n - i) - (n - j)] * b[n - j] * x**i

print(collect(c, x))

c = 0
for n_minus_i in range(n + 1):
    i = n - n_minus_i
    for n_minus_j in range(n - i + 1):
        j = n - n_minus_j
        c += a_rev[(n - i) - (n - j)] * b[n - j] * x**i

print(collect(c, x))
