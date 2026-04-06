print('#' * 40)
print('KALKULATOR INTERAKTIF SEDERHANA')
print('#' * 40)
while True:
    print('\nPilihan operasi: ')
    print('1. Tambah')
    print('2. Kurang')
    print('3. Bagi')
    print('4. Kali')
    print('5. Keluar')
    pilihan = input('Masukkan pilihan (1/2/3/4/5): ')
    if pilihan == '5':
        break
    elif pilihan in ['1', '2', '3', '4']:
        num1 = float(input('Masukkan angka pertama: '))
        num2 = float(input('Masukkan angka kedua: '))
        if pilihan == '1':
            print(f'{num1} + {num2} = {num1 + num2}')
        elif pilihan == '2':
            print(f'{num1} - {num2} = {num1 - num2}')
        elif pilihan == '3':
            if num2 == 0:
                print('Error: Pembagian dengan nol')
            else:
                print(f'{num1} / {num2} = {num1 / num2}')
        elif pilihan == '4':
            print(f'{num1} * {num2} = {num1 * num2}')
    else:
        print('Pilihan tidak valid')
