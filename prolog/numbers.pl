% my attempt at making a numbers calculator that works both ways (572943 -> 1bad0fcabc1ebdce and 1bad0fcabc1ebdce -> 572943)
% this doesn't work at all

:- use_module(library(lists)).
:- use_module(library(dif)).
% for swi prolog
% :- use_module(library(clpfd)).
% for scryer prolog
:- use_module(library(clpz)).

replace_single(X, Y, X, Y).
replace_single(X, Y, Y, Y).
replace_single(X, Y, A, A) :- A #\= X.
replace(X, Y, [], []).
replace(X, Y, [A|As], [B|Bs]) :-
  replace_single(X, Y, A, B),
  replace(X, Y, As, Bs).

replace2_single(X, Y, X, Y).
replace2_single(X, Y, A, A) :- A #\= X, A #\= Y.
replace2(X, Y, [], []).
replace2(X, Y, [A|As], [B|Bs]) :-
  replace2_single(X, Y, A, B),
  replace2(X, Y, As, Bs).

same_len([], []).
same_len([A|As], [B|Bs]) :- same_len(As, Bs).

digit(0, 48).
digit(1, 49).
digit(2, 50).
digit(3, 51).
digit(4, 52).
digit(5, 53).
digit(6, 54).
digit(7, 55).
digit(8, 56).
digit(9, 57).
to_digits_reverse(X, [A]) :-
  X #\= 0, digit(X, A).
to_digits_reverse(X, [A|As]) :-
  Div #\= 0, digit(Rem, A),
  X #= Div * 10 + Rem,
  to_digits_reverse(Div, As).
to_digits(0, [48]).
to_digits(X, Y) :-
  write(X),
  write(" "),
  write(Y),
  write("\n"),
  X #\= 0,
  to_digits_reverse(X, Z),
  reverse(Z, Y)
  .

append_digits(X, Y, Z) :-
  to_digits_reverse(X, XX),
  to_digits_reverse(Z, ZZ),
  append(YY, XX, ZZ),
  to_digits_reverse(Y, YY)
  .
%append_digits(A, B, C) :- B #\= 0, append_digits1(A, B, C).

%append_digits(1, A, B) :- B #= 
%digit_count(A, AC),
  %digit_count(B, BC),
  %digit_count(X, XC),
  %XC #= AC + BC,
  %to_digits(X, XX),
  %to_digits(A, AA),
  %to_digits(B, BB),
  %write(B),
  %write("\n"),
  %append(AA, BB, XX),
  %write(BB),
  %write(XX).

reverse_digits(X, Y) :-
  to_digits_reverse(X, StrX),
  reverse(StrX, StrY),
  to_digits_reverse(Y, StrY).

digit_char(N, C) :- C #= N + 48.

replace_digits(X, Y, A, B) :-
  to_digits_reverse(A, StrA),
  digit_char(X, X1), digit_char(Y, Y1),
  replace2(X1, Y1, StrA, StrB),
  to_digits_reverse(B, StrB).

solve(A, B) :-
  replace2(51, 102, Y6, B),
  replace2(52, 101, Y5, Y6),
  replace2(57, 100, Y4, Y5),
  replace2(50, 99, Y3, Y4),
  replace2(55, 98, Y2, Y3),
  replace2(53, 97, Y1, Y2),
  to_digits(X9, Y1),
  append_digits(X8, 24, X9),
  write("\n"),
  write(X8),
  append_digits(17, X7, X8),
  append_digits(2, A, X1),
  append_digits(X1, 91, X2),
  X3 #= X2 * 5,
  append_digits(X3, 6, X4),
  reverse_digits(X4, X5),
  replace_digits(2, 3, X5, X6),
  X7 #= X6 * 9
  .
  % atom_codes(B, Y7).

main :-
  % solve(572943, X1), atom_codes(X, X1),
  solve(X, [49,98,97,100,48,102,99,97,98,99,49,101,98,100,99,101]),
  write(X),
  halt.

:- initialization(main).

