#include <iostream>
#include <string>
#include <algorithm>

using namespace std;


int main()
{
    string in;
    cin >> in;

    size_t idx = 0, sum = 0;
    for_each(in.begin(), in.end(), [&](char a) { sum += (a-'0')*(a==in[++idx % in.length()]); });
    cout << "P1: " << sum << endl;

    idx = 0, sum = 0;
    for_each(in.begin(), in.end(), [&](char a) { sum += (a-'0')*(a==in[(idx++ + in.length()/2) % in.length()]); });
    cout << "P2: " << sum << endl;
}