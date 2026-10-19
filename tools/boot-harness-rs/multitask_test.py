"""Multiple instances in the normal F5 desktop; no fixture-only app behavior."""
def exercise(launch, click, key, note, focus, close, dialog, choose_close, color, measure):
    for word in ('one', 'two', 'three'):
        launch(1)
        note('')
        focus()
        for char in word:
            key(char)
        note(word, dirty=True)
    measure("three")
    # Admission at capacity preserves all existing instances and their drafts.
    launch(1)
    note('three', dirty=True)
    for index, word in enumerate(('one', 'two', 'three')):
        click(90, 24 + 384 + index * 32 + 16)
        note(word, dirty=True)
    # Chrome's close requests LOSS, and cannot silently discard a draft.
    click(190, 24 + 384 + 16)
    note('one', dirty=True)
    # App-owned close can explicitly discard. Survivors retain their own state.
    for index, word in enumerate(('one', 'two', 'three')):
        click(90, 24 + 384 + index * 32 + 16)
        note(word, dirty=True)
        close()
        dialog(0, 'UNSAVED NOTE')
        choose_close(2)
    color('101827')
    # Reused process slots are new instances, with no previous draft or ticket.
    launch(1)
    note('')
    close()
    color('101827')
    print('PASS: three Notes instances, independent drafts, switch, capacity refusal, protected close and slot reuse')
