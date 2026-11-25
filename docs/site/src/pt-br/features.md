[English](../features.md) · [Polski](../pl/features.md) · [Deutsch](../de/features.md) · [Español](../es/features.md) · [Français](../fr/features.md) · [Italiano](../it/features.md) · [Nederlands](../nl/features.md) · **Português (Brasil)** · [Українська](../uk/features.md)

# Recursos

## Jogos

Quatorze jogos em seis famílias, um só programa, um só conjunto de teclas:

- **Klondike**: comprar 1 ou comprar 3.
- **Spider**: 1, 2 ou 4 naipes.
- **FreeCell**.
- **Pirâmide**: relaxada ou clássica.
- **Família Yukon**: Yukon, Paciência russa, Alasca, Moosehide e Australiana.
- **Golf**: limpar sete colunas sobre o descarte, uma posição acima ou abaixo de cada vez.

O menu de jogos organiza os jogos sob títulos de família, marca com uma estrela os populares, guarda seus favoritos em uma lista própria e encontra um jogo enquanto você digita o nome dele. As regras de cada jogo estão na janela de ajuda (`i`, "Como jogar").

## Distribuições: sementes, jogos vencíveis e dificuldade

Cada distribuição tem uma **semente**. Copie-a com `c` para repetir a distribuição ou compartilhá-la.

- **Somente distribuições vencíveis.** Um novo jogo pode esperar por uma distribuição que o resolvedor integrado tenha comprovado como vencível. O resolvedor conhece cada carta, então "vencível" significa que a distribuição pode ser vencida, não que seja fácil.
- **Novo jogo com semente.** Escolha primeiro o jogo, depois de onde vem a semente: digitar uma, escolher uma das distribuições que você já jogou (a mais recente primeiro, com o resultado e o tempo), deixar o jogo sortear uma delas ou pegar uma das **distribuições selecionadas**.
- **Cinco níveis de dificuldade.** Cada jogo oferece três distribuições selecionadas por nível, de muito fácil a muito difícil. Todas são comprovadamente vencíveis. Os níveis vêm de medições: quanto trabalho o resolvedor precisou, quão longa é a linha vencedora e se seguir apenas as dicas basta para vencer a distribuição.

## Ajuda durante o jogo

- **Painel guia** (`g`): o objetivo do jogo, as teclas que funcionam agora e o que a carta em foco pode fazer. A partir de 130 colunas, ele fica ao lado da mesa, com um botão **Novo jogo** no topo; em terminais mais estreitos, abre em uma janela.
- **Painel de movimentos** (`v`): cada jogada que você pode fazer agora, com sua tecla, e o motivo quando uma jogada não é possível.
- **Dicas** (`h`): o jogo marca uma jogada na mesa e a nomeia. As dicas usam apenas o que você pode ver: nunca olham as cartas viradas para baixo nem o monte.
- **Jogadas rápidas**: `a` envia as cartas sob o cursor para o melhor destino, `e` faz o mesmo com a carta do topo do descarte a partir de qualquer ponto da mesa, e o clique duplo funciona como `a`. As jogadas rápidas podem ser desligadas nas configurações ou nas opções rápidas (`o`), se você preferir posicionar cada carta sozinho.
- **Finalização automática**: quando a vitória é certa, as jogadas restantes são feitas por você. Pode ser desligada.
- **Desfazer e refazer** (`u`, `r`), com uma opção para jogar sem desfazer.

## Teclado e mouse

Cada ação funciona com o teclado e com o mouse. Clique em uma carta para pegá-la e em um destino para soltá-la, dê clique duplo para uma jogada rápida, arraste uma carta até o destino e clique com o botão direito para devolvê-la. Os botões de janelas e painéis mostram a tecla, e a aba ativa de uma janela nunca parece o botão em foco. A janela de ajuda (`?`) lista cada tecla; ela também pode ficar acoplada na lateral da tela (`d`).

## Configurações

As configurações têm três abas:

- **Populares** abre primeiro e reúne as configurações mais alteradas: idioma, baixa visão, tema, tamanho das cartas, verso da carta, volume, aura e finalização automática.
- **Áudio e visual** reúne tudo sobre aparência e som: cores, tela cheia e tamanho da janela, tema, visão de cores, estilos e tons das cartas, a mesa, a aura, a fonte do console, o som e a prioridade do processo.
- **Jogabilidade** reúne as ajudas: marcas de destino, os painéis guia e de movimentos, finalização automática e jogadas rápidas.

Uma prévia ao lado da lista mostra uma mudança antes de você mantê-la.

## Cartas e mesa

- **Seis estilos de faces**: Internacional, Francês, Alemão, Italiano, Espanhol e Suíço, cada um com seus próprios símbolos de naipe.
- **Dois estilos de verso** (Treliça e Ornamento) e cores de verso vindas do tema, Monokai ou Dracula.
- **Tom do fundo da carta**: o do tema, branco, marfim, cinza claro ou escuro.
- **Mesa**: a cor do tema, feltro verde, azul, vermelho, cinza, marrom ou preto, e a superfície lisa, de feltro ou pautada.
- **Tamanho das cartas**: aumente as cartas com `]` e diminua com `[`; `0` volta ao tamanho automático que cabe na janela. Colunas longas se aproximam para que a última carta fique sempre totalmente visível, e as cartas só diminuem quando aproximar a coluna não basta.

## Temas e acessibilidade

Cinco temas: **Escuro** e **Claro** para o dia a dia, **Alto contraste** para baixa visão, **Monokai** e **Dracula**. Cada um é mostrado aqui na mesma posição.

![O tema Escuro](img/pt/theme-dark.svg)

![O tema Claro](img/pt/theme-light.svg)

![O tema Alto contraste](img/pt/theme-high-contrast.svg)

![O tema Monokai](img/pt/theme-monokai.svg)

![O tema Dracula](img/pt/theme-dracula.svg)

- **Baixa visão.** Uma única chave, oferecida na primeira execução e nas configurações: contraste de texto de pelo menos 7:1 em qualquer tema, as maiores cartas que couberem, um halo grosso, valor e naipe em negrito, marcas de foco e de destino maiores e, no console clássico do Windows, uma fonte maior. Cada parte continua ajustável, e desligar o modo restaura suas configurações anteriores.
- **Visão de cores.** Modos para protanopia, deuteranopia, tritanopia e monocromático mudam as cores para que o que você precisa distinguir continue distinguível. O **baralho de quatro cores** dá a cada naipe a sua cor: espadas pretas, copas vermelhas, ouros azuis e paus verdes.
- **A cor nunca é o único sinal.** Os naipes são símbolos (♠ ♥ ♦ ♣) ao lado do valor, o item em foco leva uma marca, as cartas pegas e os destinos têm marcas próprias, os resultados trazem ✓ ou ✗, e uma linha sob a mesa diz em palavras onde está o cursor e o que ele pode fazer.

## Estatísticas

A tela de estatísticas mostra os resultados de cada jogo (vitórias, sequências, melhor tempo e menos jogadas) e um histórico de cada distribuição, a mais recente primeiro e no seu horário local. Pelo histórico você pode jogar uma semente de novo, copiá-la ou pedir ao resolvedor que verifique se uma distribuição era vencível.

## Console do Windows

No Windows, um jogo iniciado com um clique duplo, ou com `oxysol -n`, tem a própria janela de console. Ele abre em tela cheia por padrão, e em modo janela você pode escolher o tamanho nas configurações: 130 × 40, 195 × 60 ou 260 × 80. Um jogo iniciado em um console que já está aberto deixa esse console como está. No console clássico do Windows, Ctrl+Mais, Ctrl+Menos e Ctrl+0 mudam o tamanho da fonte do console; outros terminais ampliam com as próprias teclas.

## Idiomas

O jogo está disponível em nove idiomas: inglês, polonês, alemão, espanhol, francês, italiano, holandês, português do Brasil e ucraniano. Escolha um com o seletor de bandeiras no canto superior esquerdo do menu principal, ou nas configurações; a mudança vale na hora. Na primeira execução, o jogo segue o idioma do seu sistema. Este site está disponível nos mesmos nove idiomas.

## O que ainda não é

oXySol é uma versão de pré-lançamento. Espere algumas arestas e conte para nós no
[rastreador de problemas](https://github.com/beoArcha/oXyFlow.Solitaire/issues).
