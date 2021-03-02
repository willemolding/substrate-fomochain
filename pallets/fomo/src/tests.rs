use crate::{mock::*};
use frame_support::{assert_ok, assert_noop};

// single participants tests

#[test]
fn can_spend_balance_and_buy_initial_ticket() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(0), INITIAL_BALANCE);
        assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), PRICE_INCREMENT));
        assert_eq!(Balances::free_balance(0), INITIAL_BALANCE - PRICE_INCREMENT);
    });
}

#[test]
fn only_price_deducted_when_greater_max_sent() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(0), INITIAL_BALANCE);
        assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), INITIAL_BALANCE));
        assert_eq!(Balances::free_balance(0), INITIAL_BALANCE - PRICE_INCREMENT);
    });
}

#[test]
fn price_increases_by_increment() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(0), INITIAL_BALANCE);
        assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), PRICE_INCREMENT));
        assert_eq!(Balances::free_balance(0), INITIAL_BALANCE - PRICE_INCREMENT);
        assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), 2*PRICE_INCREMENT));
        assert_eq!(Balances::free_balance(0), INITIAL_BALANCE - PRICE_INCREMENT - 2*PRICE_INCREMENT);
    });
}

#[test]
fn can_buy_up_to_final_block() {
    new_test_ext().execute_with(|| {
    	for block_number in 0..BLOCKS_TO_WIN {
    		run_to_block(block_number.into());
        	assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), INITIAL_BALANCE));
    	}
    });
}

#[test]
fn cannot_buy_in_final_block() {
    new_test_ext().execute_with(|| {
    	run_to_block(BLOCKS_TO_WIN.into());
        assert_noop!(
			FOMOModule::buy_ticket(Origin::signed(0), INITIAL_BALANCE),
			pallet_fomo::Error::<Test>::GameIsOver,
		);
    });
}


#[test]
fn cannot_claim_before_end() {
    new_test_ext().execute_with(|| {
    	assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), INITIAL_BALANCE));
        assert_noop!(
			FOMOModule::claim(Origin::signed(0)),
			pallet_fomo::Error::<Test>::GameIsNotOver,
		);
    });	
}

#[test]
fn winner_can_claim_and_gets_funds_back() {
    new_test_ext().execute_with(|| {
    	assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), INITIAL_BALANCE));
    	run_to_block(BLOCKS_TO_WIN.into());
    	assert_ok!(FOMOModule::claim(Origin::signed(0)));
    	assert_eq!(Balances::free_balance(0), INITIAL_BALANCE);
    });	
}

// tests with two participants

#[test]
fn two_players_buy_tickets_only_last_can_claim() {
    new_test_ext().execute_with(|| {
    	assert_ok!(FOMOModule::buy_ticket(Origin::signed(0), INITIAL_BALANCE));
    	assert_ok!(FOMOModule::buy_ticket(Origin::signed(1), INITIAL_BALANCE));
    	run_to_block(BLOCKS_TO_WIN.into());
        assert_noop!(
			FOMOModule::claim(Origin::signed(0)),
			pallet_fomo::Error::<Test>::ClaimerIsNotLeader,
		);
    	assert_ok!(FOMOModule::claim(Origin::signed(1)));
    	assert_eq!(Balances::free_balance(0), INITIAL_BALANCE - PRICE_INCREMENT);
    	assert_eq!(Balances::free_balance(1), INITIAL_BALANCE + PRICE_INCREMENT);
    });	
}
