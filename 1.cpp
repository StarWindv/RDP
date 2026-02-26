#include <iostream>
#include <chrono>
using namespace std;
using namespace chrono;

// 纯递归斐波那契，无任何优化
long long fib(int n) {
    if (n <= 2)
        return 1;
    return fib(n - 1) + fib(n - 2);
}

int main() {
    int n = 30;

    auto start = high_resolution_clock::now();

    long long result = fib(n);

    auto end = high_resolution_clock::now();

    auto duration = duration_cast<milliseconds>(end - start).count();

    cout << "fib(" << n << ") = " << result << endl;
    cout << "纯递归耗时: " << duration << " ms" << endl;

    return 0;
}
