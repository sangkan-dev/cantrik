print('SELAMAT DATANG DI GAME TEBAK ANGKA!')
import random
angka_rahasia = random.randint(1, 10)
while True:
    tebakan = int(input('Tebak angka antara 1-10: '))
    if tebakan == angka_rahasia:
        print('SELAMAT! Anda berhasil menebak angka rahasia.')
        break
    elif tebakan < angka_rahasia:
        print('Angka yang Anda tebak terlalu kecil.')
    else:
        print('Angka yang Anda tebak terlalu besar.')
