-- =================================================
-- TESTE DE ESTRESSE: LUA-RUST VM
-- =================================================

-- 1. Teste de Globais e Aritmética
g_total_execs = 0
g_target = 100

print("--- Iniciando Teste de Globais ---")
g_total_execs = (10 + 5 * 2) / 2 -- Deve ser 10
print("Resultado Aritmetica (esperado 10):", g_total_execs)

-- 2. Teste de Recursão (Fibonacci)
-- Testa CallFrame, LoadLocal, Sub e Return
function fib(n)
    if n == 0 then
        return 0
    end
    if n == 1 then
        return 1
    end
    return fib(n - 1) + fib(n - 2)
end

print("--- Teste de Recursao ---")
local f5 = fib(5)
local f10 = fib(10)
print("Fibonacci(5)  (esperado 5):", f5)
print("Fibonacci(10) (esperado 55):", f10)

-- 3. Teste de Loops e Break
-- Testa While, Jump, JumpIfFalse e Break
print("--- Teste de Loop e Break ---")
local contador = 0
local acumulador = 0

while true do
    if contador == 10 then
        print("Saindo do loop com break...")
        break
    end

    acumulador = acumulador + contador
    contador = contador + 1
end
print("Soma de 0 a 9 (esperado 45):", acumulador)

-- 4. Teste de Escopo e Shadowing
-- Testa se a VM limpa a pilha corretamente (Pop)
print("--- Teste de Escopo ---")
local x = 10
print("x original:", x)

function test_scope()
    local x = 50 -- Shadowing da local externa
    print("x dentro da funcao (esperado 50):", x)

    if true then
        local x = 100 -- Shadowing da local da função
        print("x dentro do if (esperado 100):", x)
    end

    print("x apos o if (esperado 50):", x)
end

test_scope()
print("x fora de tudo (esperado 10):", x)

-- 5. Teste de Funções como "Cidadãs de Primeira Classe"
-- (Carregar função global e passar como valor)
function dobro(n)
    return n * 2
end

function aplicar(f_nome, val)
    -- Nota: Atualmente seu compilador busca 'dobro' como global
    return dobro(val)
end

print("--- Teste de Chamada Indireta ---")
print("Aplicar dobro em 25 (esperado 50):", aplicar("dobro", 25))

-- 6. Teste de Nil e Retorno Implicito
function sem_retorno()
    local t = 1 + 1
end

print("--- Teste de Retorno Nil ---")
local resultado_nil = sem_retorno()
print("Resultado de funcao vazia (esperado nil):", resultado_nil)

-- 7. Algoritmo de Collatz (Testa tudo junto)
-- Se n for par -> n/2 | Se impar -> 3n + 1
print("--- Sequencia de Collatz (n=6) ---")
local n = 6
while n == 1 == false do -- Simulando n != 1
    print("Atual:", n)

    -- Checar se é par (usando lógica manual pois não temos %)
    local metade = n / 2
    -- Se metade é inteiro (nossa VM usa f64, isso é um teste de precisão)
    -- Aqui vamos apenas simular um passo para não complicar
    if n == 6 then
        n = 3
    else
        if n == 3 then
            n = 10
        else
            if n == 10 then
                n = 5
            else
                if n == 5 then
                    n = 16
                else
                    if n == 16 then
                        n = 8
                    else
                        if n == 8 then
                            n = 4
                        else
                            if n == 4 then
                                n = 2
                            else
                                if n == 2 then
                                    n = 1
                                end
                            end
                        end
                    end
                end
            end
        end
    end
end

print("Fim da sequencia: ", n)
print("=================================================")
print("TESTE CONCLUIDO COM SUCESSO")
