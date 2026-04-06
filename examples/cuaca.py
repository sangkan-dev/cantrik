import requests
print('SELAMAT DATANG DI APLIKASI CUACA!')
while True:
    kota = input('Masukkan nama kota: ')
    api_url = f'https://api.openweathermap.org/data/2.5/weather?q={kota}&appid=YOUR_API_KEY'
    response = requests.get(api_url)
    if response.status_code == 200:
        data = response.json()
        cuaca = data['weather'][0]['main']
        suhu = data['main']['temp']
        print(f'Cuaca di {kota}: {cuaca}, Suhu: {suhu} Kelvin')
    else:
        print('Gagal mendapatkan data cuaca.')
