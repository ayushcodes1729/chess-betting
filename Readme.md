Given your diagram, treasury is global and not tied to each match.
So:

Create one global treasury PDA at program init.

Store its bump in a Config account (not in Match).

All matches will send leftover funds there.

When you need to withdraw, run a withdraw_from_treasury instruction with you as signer and treasury PDA as source.