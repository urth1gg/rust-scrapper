use scraper::{Html, Selector};
use std::convert::TryInto;
use regex::Regex;
use std::collections::HashSet;
use crate::records_data::RecordsData;
use crate::data;
pub struct Extractor {
    pub html: String,
}
#[derive(Debug)]
pub struct CompanyInfo {
    pub company: String,
    pub link: String,
}

pub struct CompanyContactDetails {
    pub phone: String,
    pub website: String,
}

impl Extractor {
    pub fn new(html: String) -> Self {
        Self { html }
    }

    pub fn set_html(&mut self, html: String) {
        self.html = html;
    }

    pub fn get_company_info(&self) -> Vec<CompanyInfo> {
        let document = Html::parse_document(&self.html);
        let profile_selector = Selector::parse(".searchprofile").unwrap();
        let mut company_info_list = Vec::new();

        for profile in document.select(&profile_selector) {
            if let Some(link_element) = profile
                .select(&Selector::parse(".more-info > a").unwrap())
                .next()
            {
                let link = link_element
                    .value()
                    .attr("href")
                    .unwrap_or_default()
                    .to_string();
                let company = profile
                    .select(&Selector::parse("h3 > a").unwrap())
                    .next()
                    .map(|e| e.inner_html())
                    .unwrap_or_default();

                company_info_list.push(CompanyInfo { company, link });
            }
        }

        company_info_list
    }

    pub fn get_company_details(&self) -> CompanyContactDetails {
        let document = Html::parse_document(&self.html);

        // Selector for phone number
        let phone_selector = Selector::parse(".member-contact a[href^='tel:']").unwrap();
        let phone = document
            .select(&phone_selector)
            .next()
            .map(|e| e.inner_html())
            .unwrap_or_default();

        // Selector for website URL
        let website_selector = Selector::parse(
            ".member-contact a[href^='http://'], .member-contact a[href^='https://']",
        )
        .unwrap();
        let website = document
            .select(&website_selector)
            .next()
            .map(|e| e.value().attr("href").unwrap_or_default())
            .unwrap_or_default();

        CompanyContactDetails {
            phone,
            website: website.to_string(),
        }
    }

    pub fn find_contact_us_link(&self) -> Option<String> {
        let document = Html::parse_document(&self.html);
        let contact_us_selector = Selector::parse("a[href*='contact']").unwrap();
        let contact_us_link = document
            .select(&contact_us_selector)
            .next()
            .map(|e| e.value().attr("href").unwrap_or_default())
            .unwrap_or_default();

        if contact_us_link.is_empty() {
            None
        } else {
            Some(contact_us_link.to_string())
        }
    }

    pub fn find_emails_by_regex(&self) -> String {
        let email_regex = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        
        let mut emails = Vec::new();
        let mut seen = HashSet::new();
    
        for mat in email_regex.find_iter(&self.html) {
            let email = mat.as_str().to_string();
            if seen.insert(email.clone()) {
                emails.push(email);
            }
        }

        emails.join(", ")
    }

    pub fn get_company_info_houzz(&self) -> Vec<CompanyInfo> {

        let document = Html::parse_document(&self.html);
        let profile_selector = Selector::parse(".hz-pro-search-results").unwrap();
        let link_selector = Selector::parse(".hz-pro-search-results__item a").unwrap();
        let company_selector = Selector::parse("span[itemprop='name']").unwrap();
        let mut company_info_list = Vec::new();
    
        for profile in document.select(&profile_selector) {
            for link_element in profile.select(&link_selector) {
                let link = link_element
                    .value()
                    .attr("href")
                    .unwrap_or_default()
                    .to_string();
    
                if let Some(company_element) = link_element.select(&company_selector).next() {
                    let company = company_element.inner_html();
    
                    company_info_list.push(CompanyInfo { company, link });
                }
            }
        }
    
        company_info_list
    }

    pub fn get_company_details_houzz(&self) -> CompanyContactDetails {
        let document = Html::parse_document(&self.html);

        // Selector for phone number
        let phone_selector = Selector::parse("#business > div > div:nth-child(2) > p").unwrap();
        let phone = document
            .select(&phone_selector)
            .next()
            .map(|e| e.inner_html())
            .unwrap_or_default();

        // Selector for website URL
        let website_selector = Selector::parse(
            "div[data-component='Website'] span[font-size='smallPlus,medium']",
        )
        .unwrap();
        let website = document
            .select(&website_selector)
            .next()
            .map(|e| e.inner_html())
            .unwrap_or_default();

        CompanyContactDetails {
            phone,
            website: website.to_string(),
        }
    }
}

impl TryInto<RecordsData> for CompanyContactDetails {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<RecordsData, Self::Error> {
        // Assuming you can derive the `company` field from somewhere in your context

        Ok(RecordsData {
            id: 0,                // Assuming this will be set by the database
            records_html_id: 0,   // You need to provide this value from your context
            email: String::new(), // Default value or derive from context
            phone: self.phone.to_string(),
            website: self.website.to_string(),
            contact_us_link: Some(String::new()), // Default value or derive from context
        })
    }
}

impl CompanyContactDetails {}
#[cfg(test)]
mod tests {
    use super::*;
    static HTML: &str = r#"
    <div class="searchprofile col-md-5 col-xs-12">
    <div class="logo">
                <a href="https://landscapeontario.com/member/figure-4-design-consultancy" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">
                <img class="img-responsive center-block" src="https://landscapeontario.com/thumbnailer.php?imgWH=200&amp;image=/assets/1552092364.logo.006842.png" alt="Figure 4 Landscapes">
            </a>
            </div>
    <h3 class="heading text-center">
        <a href="https://landscapeontario.com/member/figure-4-design-consultancy" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">Figure 4 Landscapes</a>
    </h3>
    <div class="info">
        <div class="address">
            <i class="fa fa-fw fa-map-marker" aria-hidden="true"></i>Etobicoke
        </div>
        <div class="phone">
            <i class="fa fa-fw fa-phone" aria-hidden="true"></i>
            <a href="tel:+416-803-7650" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">416-803-7650</a>
        </div>
        <div class="more-info">
            <a href="https://landscapeontario.com/member/figure-4-design-consultancy" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">
                <button class="btn btn-success btn-sm btn-block outline">More Info</button>
            </a>
        </div>
    </div>
    <div class="description"></div>
    </div>
    <div class="searchprofile col-md-5 col-xs-12">
    <div class="logo">
                <a href="https://landscapeontario.com/member/figure-4-design-consultancy" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">
                <img class="img-responsive center-block" src="https://landscapeontario.com/thumbnailer.php?imgWH=200&amp;image=/assets/1552092364.logo.006842.png" alt="Figure 4 Landscapes">
            </a>
            </div>
    <h3 class="heading text-center">
        <a href="https://landscapeontario.com/member/figure-4-design-consultancy" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">Figure 4 Landscapes</a>
    </h3>
    <div class="info">
        <div class="address">
            <i class="fa fa-fw fa-map-marker" aria-hidden="true"></i>Etobicoke
        </div>
        <div class="phone">
            <i class="fa fa-fw fa-phone" aria-hidden="true"></i>
            <a href="tel:+416-803-7650" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">416-803-7650</a>
        </div>
        <div class="more-info">
            <a href="https://landscapeontario.com/member/figure-4-design-consultancy" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">
                <button class="btn btn-success btn-sm btn-block outline">More Info</button>
            </a>
        </div>
    </div>
    <div class="description"></div>
    </div>
    "#;

    static HTML_CONTACT: &str = r#"
    <div class="member-contact">
        <h2>Member Since 2019</h2>
        <div>
            <h4>
                <i class="fa fa-globe" aria-hidden="true"></i> 
                <a href="https://www.mdrlandscapes.com/" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">mdrlandscapes.com</a>
            </h4>
            <h4>
                <i class="fa fa-phone" aria-hidden="true"></i> 
                <a href="tel:+416-948-2966" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">416-948-2966</a>
            </h4>
        </div>
    </div>
"#;

    static HTML_CONTACT_NO_WEBSITE: &str = r#"
    <div class="member-contact">
        <h2>Member Since 2019</h2>
        <div>
            <h4>
                <i class="fa fa-phone" aria-hidden="true"></i> 
                <a href="tel:+416-948-2966" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">416-948-2966</a>
            </h4>
        </div>
    </div>
"#;

    static HTML_CONTACT_NO_PHONE: &str = r#"
    <div class="member-contact">
        <h2>Member Since 2019</h2>
        <div>
            <h4>
                <i class="fa fa-globe" aria-hidden="true"></i> 
                <a href="https://www.mdrlandscapes.com/" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">mdrlandscapes.com</a>
            </h4>
        </div>
    </div>
"#;

    static HTML_CONTACT_FIND_PHONE: &str = r#"
    <header id="main-header" class="et_nav_text_color_dark" data-height-onload="146" data-height-loaded="true" data-fixed-height-onload="146">
    <div class="container clearfix">
                    <a href="https://letslandscape.ca/">
            <img src="https://letslandscape.ca/images/Lets-Landscape-Together.png" alt="Let's Landscape Together" id="logo" data-actual-width="328" data-actual-height="236">
        </a>

        <div id="et-top-navigation" style="padding-left: 196px;">
            <nav id="top-menu-nav" class="right-second-menu">
            <ul id="top-menu" class="nav"><li id="menu-item-684" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-home current-menu-item page_item page-item-665 current_page_item menu-item-684"><a title="Burlington Landscaping by Let’s Landscape Together" href="https://letslandscape.ca/" aria-current="page">Home</a></li>
<li id="menu-item-55" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-55"><a title="About Let’s Landscape Together" href="https://letslandscape.ca/about-us/">About Us</a></li>
<li id="menu-item-54" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-54"><a title="Landscaping Services" href="https://letslandscape.ca/services/">Services</a></li>
<li id="menu-item-53" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-53"><a title="Landscaping Photo Gallery" href="https://letslandscape.ca/gallery/">Landscaping Photo Gallery</a></li>
<li id="menu-item-52" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-52"><a title="Contact us about your landscaping project today" href="https://letslandscape.ca/contact-us/">Contact Us</a></li>
</ul>					</nav>
            
                                <div id="et_top_search">
                <span id="et_search_icon"></span>
                <form role="search" method="get" class="et-search-form et-hidden" action="https://letslandscape.ca/">
                <input type="search" class="et-search-field" placeholder="Search …" value="" name="s" title="Search for:">						</form>
            </div>
            
            <div id="et_mobile_nav_menu">
        <div class="mobile_nav closed">
            <span class="select_page">Select Page</span>
            <span class="mobile_menu_bar mobile_menu_bar_toggle"></span>
        <ul id="mobile_menu" class="et_mobile_menu"><li id="menu-item-684" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-home current-menu-item page_item page-item-665 current_page_item menu-item-684 et_first_mobile_item"><a title="Burlington Landscaping by Let’s Landscape Together" href="https://letslandscape.ca/" aria-current="page">Home</a></li>
<li id="menu-item-55" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-55"><a title="About Let’s Landscape Together" href="https://letslandscape.ca/about-us/">About Us</a></li>
<li id="menu-item-54" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-54"><a title="Landscaping Services" href="https://letslandscape.ca/services/">Services</a></li>
<li id="menu-item-53" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-53"><a title="Landscaping Photo Gallery" href="https://letslandscape.ca/gallery/">Landscaping Photo Gallery</a></li>
<li id="menu-item-52" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-52"><a title="Contact us about your landscaping project today" href="https://letslandscape.ca/contact-us/">Contact Us</a></li>
<li class="menu-item menu-item-type-post_type menu-item-object-page menu-item-58"><a title="Book your landscaping consultation today!" href="https://letslandscape.ca/request-a-consultation/">Request A Consultation</a></li>
</ul></div>
    </div>


        </div> <!-- #et-top-navigation -->
    </div> <!-- .container -->
<div id="mobile-nav-container">
<nav id="mobile-navigation" style="text-align: center;">
<a href="https://letslandscape.ca/#lets-landscape-together" class="mobile-nav-button" style="background-color:#8CC63F;" title="Home - Burlington Landscape Company">HOME</a>
<a href="https://letslandscape.ca/about-us/#landscaping-company" class="mobile-nav-button" style="background-color:#FFC20D;" title="About our Landscaping Company">ABOUT US</a>
<a href="https://letslandscape.ca/services/#landscaping-services" class="mobile-nav-button" style="background-color:#C1272D;" title="Landscaping Services Near Me">SERVICES</a>
<a href="https://letslandscape.ca/gallery/#landscaping-photos" class="mobile-nav-button" style="background-color:#F5821F;" title="Landscaping Photo Gallery">GALLERY</a>
<a href="https://letslandscape.ca/contact-us/#burlington-landscaping" class="mobile-nav-button" style="background-color:#00A2DD;" title="Contact us for a landscaping consultation">CONTACT US</a>
</nav>
</div>
</header>    
"#;

    static HTML_CONTACT_FIND_EMAIL: &str = r#"
    <div class="member-contact">
        <h2>Member Since 2019</h2>
        <div>
            <h4>
                <i class="fa fa-globe" aria-hidden="true"></i> 
                <a href="https://www.mdrlandscapes.com/" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">mdrlandscapes.com</a>
            </h4>
            <h4>
                <i class="fa fa-phone" aria-hidden="true"></i> 
                <a href="tel:+416-948-2966" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">416-948-2966</a>
            </h4>
        </div>
        djolecs97@gmail.com
        ikariam1234@youtube.com
        test123@blabla.com
    </div>
"#;

    static HTML_CONTACT_FIND_EMAIL_SINGLE: &str = r#"
    <div class="member-contact">
        <h2>Member Since 2019</h2>
        <div>
            <h4>
                <i class="fa fa-globe" aria-hidden="true"></i> 
                <a href="https://www.mdrlandscapes.com/" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">mdrlandscapes.com</a>
            </h4>
            <h4>
                <i class="fa fa-phone" aria-hidden="true">djoko@bestbuy.org</i> 
                <a href="tel:+416-948-2966" data-feathr-click-track="true" data-feathr-link-aids="[&quot;5d9e4d26514f59f11c68a738&quot;]">416-948-2966</a>
            </h4>
        </div>
    </div>
"#;



    #[test]
    fn should_extract_company_info_houzz() {
        let html_houzz = data::test_generate_houzz_html();
        let extractor = Extractor::new(html_houzz.to_string());
        let company_info = extractor.get_company_info_houzz();
        println!("{:?}", company_info);
        assert_eq!(company_info.len(), 6);
    }

    #[test]
    fn should_extract_company_details_houzz(){
        let html_record_houzz = data::test_generate_houzz_record_html();
        let extractor = Extractor::new(html_record_houzz.to_string());
        let company_details = extractor.get_company_details_houzz();

        assert_eq!(company_details.phone, "(905) 713-1230");
        assert_eq!(company_details.website, "www.mcfees.com");
    }
    #[test]
    fn should_extract_company_info() {
        let extractor = Extractor::new(HTML.to_string());
        let company_info = extractor.get_company_info();
        assert_eq!(company_info.len(),2);
    }

    #[test]
    fn length_should_be_2() {
        let extractor = Extractor::new(HTML.to_string());
        let company_info = extractor.get_company_info();
        assert_eq!(company_info.len(), 2);
    }

    #[test]
    fn first_company_should_be_correct() {
        let extractor = Extractor::new(HTML.to_string());
        let company_info = extractor.get_company_info();
        assert_eq!(company_info[0].company, "Figure 4 Landscapes");
        assert_eq!(
            company_info[0].link,
            "https://landscapeontario.com/member/figure-4-design-consultancy"
        );
    }

    #[test]
    fn should_have_website_and_email() {
        let extractor = Extractor::new(HTML_CONTACT.to_string());
        let company_details = extractor.get_company_details();
        assert_eq!(company_details.phone, "416-948-2966");
        assert_eq!(company_details.website, "https://www.mdrlandscapes.com/");
    }

    #[test]
    fn website_should_be_empty() {
        let extractor = Extractor::new(HTML_CONTACT_NO_WEBSITE.to_string());
        let company_details = extractor.get_company_details();
        assert_eq!(company_details.website, "");
    }

    #[test]
    fn phone_should_be_empty() {
        let extractor = Extractor::new(HTML_CONTACT_NO_PHONE.to_string());
        let company_details = extractor.get_company_details();
        assert_eq!(company_details.phone, "");
    }

    #[test]
    fn should_extract_contact_us_link(){
        let extractor = Extractor::new(HTML_CONTACT_FIND_PHONE.to_string());
        let contact_us_link = extractor.find_contact_us_link().unwrap();

        assert_eq!(contact_us_link, "https://letslandscape.ca/contact-us/");
    }    

    #[test]
    fn should_extract_emails(){
        let extractor = Extractor::new(HTML_CONTACT_FIND_EMAIL.to_string());
        let emails = extractor.find_emails_by_regex();

        assert_eq!(emails, "djolecs97@gmail.com, ikariam1234@youtube.com, test123@blabla.com");
    }

    #[test]
    fn should_extract_single_email(){
        let extractor = Extractor::new(HTML_CONTACT_FIND_EMAIL_SINGLE.to_string());
        let emails = extractor.find_emails_by_regex();

        assert_eq!(emails, "djoko@bestbuy.org");


    }
    
}
