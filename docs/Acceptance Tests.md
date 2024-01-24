# Acceptance Tests

1) Boot Program
    - Action: Either from the console or desktop, run program
    - Acceptance: UI window appears and landing screen shows

2) Exit Program
    - Action: Click a 'close' button to exit program
    - Acceptance: Window closes.
    - Note: Of course the daemon stays running, just the client closes.

3) Select "Create Patient File"
    - Action: From landing screen, select option to create patient file
    - Acceptance: User is redirected to patient creation view.

4) Create Patient File
    - Action: Fill in fields in form, and click button to save.
    - Acceptance: User is redirected to patient file view with new file record.

5) Select Existing Patient File
    - Action: From list on landing screen, select name of patient that exists in system.
    - Acceptance: User is redirected to patient file view with all information displayed properly.

6) Select Individual Record from Patient File
    - Action: From patient file view, select a record from the list.
    - Acceptance: User is redirected to view with full record information.

7) Close Individual Record View
    - Action: From individual record view, press close button.
    - Acceptance: User is redirected back to patient file.

8) Select "Add New Record" to Patient File
    - Action: Click "Add New Record" button to add new record to patient file.
    - Acceptance: User is redirected to New Record view.

9) Add New Record to Patient File
    - Action: Fill in details in form and click submission button.
    - Acceptance: User is redirected back to patient page with new record now on list.

10) Select "Update Patient Information" from Patient File
    - Action: From patient file, select button to edit patient information.
    - Acceptance: User is redirected to view of editable form of patient information.

11) Update Patient Information
    - Action: Fill in updated information and click save button.
    - Acceptance: User is redirected back to patient file view, with new information displaying.

12) Select a Provider from Patient File Provider List
    - Action: Select a provider from provider list
    - Acceptance: User is redirected to view of individual provider information

13) Remove Provider
    - Action: From the individual provider view, click the 'remove' call to action to revoke access to provider.
    - Acceptance: User is redirected back to patient file page, with provider removed from list.

14) Select "Add Provider" from Patient File
    - Action: From patient file view, select 'add provider' call-to-action.
    - Acceptance: User is redirected to form of provider information.

15) Add Provider
    - Action: Click the 'submit' button to attempt to add provider to patient file
    - Acceptance: Confirmation message and user is redirected back to patient file page with provider in list of providers.